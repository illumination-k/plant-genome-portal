# ChIP-seq / CUT&RUN pipeline

Nextflow DSL2 pipeline that turns raw ChIP-seq or CUT&RUN reads (local FASTQ or
SRA accession) into:

- `bowtie2/<sample>/*.sorted.bam` — coordinate-sorted, indexed alignments
- `markduplicates/<sample>/` — duplicate-marked BAM + Picard metrics
- `filtered/<sample>/*.filt.bam` — MAPQ + flag filtered BAM (input to peak callers)
- `bigwig/<sample>.bw` — deepTools `bamCoverage` track (CPM by default)
- `macs2/<sample>/` — MACS2 narrowPeak / broadPeak (for `assay=chipseq`)
- `seacr/<sample>/` — SEACR peak BEDs + fragment bedgraph (for `assay=cutrun`)
- `picard/<sample>/` — `CollectMultipleMetrics` (insert size, alignment summary, ...)
- `multiqc_report.html` — aggregated QC over fastp, Bowtie2, samtools, Picard

The two assays share the same trimming + alignment + filtering stack and diverge
only at peak calling, where the per-sample `assay` column picks MACS2 or SEACR.

## Layout

```
pipelines/chipseq/
  main.nf                       # entrypoint
  nextflow.config               # params, profiles, reports
  conf/
    base.config                 # default resources, retries
    test.config                 # minimal test profile
  workflows/
    chipseq.nf                  # wire-up
  modules/local/
    sra_fetch.nf                # prefetch + fasterq-dump
    fastp.nf                    # QC + adapter trim
    samtools_faidx.nf           # genome .fai + chrom.sizes
    bowtie2_build.nf            # Bowtie2 index
    bowtie2_align.nf            # alignment (assay-aware params)
    samtools_sort.nf            # SAM → sorted BAM + flagstat + stats
    picard_markduplicates.nf    # MarkDuplicates + metrics
    samtools_filter.nf          # flag / MAPQ filter
    picard_collectmultiplemetrics.nf
    deeptools_bamcoverage.nf    # bigWig track
    macs2_callpeak.nf           # ChIP-seq peak calling
    samtools_nsort.nf           # name-sort BAM (for SEACR)
    bedtools_fragbedgraph.nf    # PE fragments → bedgraph (for SEACR)
    seacr_callpeak.nf           # CUT&RUN peak calling
    multiqc.nf                  # aggregate QC report
  assets/
    multiqc_config.yml
    NO_FILE                     # sentinel for optional inputs (control BAM, etc.)
  samplesheets/samplesheet.example.csv
```

## Quick start

```bash
nextflow run pipelines/chipseq \
    -profile docker \
    --input        samplesheet.csv \
    --genome_fasta data/marpolbase/MpTak1_v7.1/MpTak1_v7.1.fa.gz \
    --macs_gsize   2.25e8 \
    --outdir       results/chipseq
```

Use `-profile singularity` on HPC. All processes pin BioContainers images, so no
local installs are required beyond Nextflow (>= 24.04) and a container runtime.

`--macs_gsize` is required when any sample is `assay=chipseq` — pass the effective
genome size for the organism (Marchantia `MpTak1_v7.1` ≈ `2.25e8`).

## Samplesheet

CSV with header `sample,group,replicate,control,assay,sra,fastq_1,fastq_2`.

| column    | required    | notes                                                                 |
| --------- | ----------- | --------------------------------------------------------------------- |
| sample    | yes         | Sample ID. Used for output filenames and MultiQC grouping.            |
| group     | optional    | Replicate group label. Defaults to `sample` if omitted.               |
| replicate | optional    | Replicate index within `group`. Defaults to `1`.                      |
| control   | optional    | `sample` ID of the input / IgG control. Empty → no-control mode.      |
| assay     | optional    | `chipseq` (default) or `cutrun`. Drives Bowtie2 params + peak caller. |
| sra       | optional    | SRA run accession (SRR/ERR/DRR). If set, `fastq_*` are ignored.       |
| fastq_1   | when no sra | Path to R1 FASTQ (gz ok).                                             |
| fastq_2   | optional    | Path to R2 for paired-end. Leave blank for single-end.                |

A sample is treated as a **control** (no peaks called) when at least one other row
references it via its `control` column. Otherwise it's an **IP** and goes to the
configured peak caller.

### Paired-end vs single-end

Layout is detected per sample and propagated as `meta.single_end`:

- **Local FASTQ**: `fastq_2` populated → paired-end; empty → single-end.
- **SRA**: detected after `fasterq-dump --split-files` from the number of FASTQ
  files emitted.

The flag drives PE/SE branches in every downstream process:

| Process               | Paired                                                   | Single                          |
| --------------------- | -------------------------------------------------------- | ------------------------------- |
| FASTP                 | `--in1` / `--in2`, adapter PE detect                     | `--in1` only                    |
| BOWTIE2_ALIGN         | `-1` / `-2`, `--no-mixed --no-discordant`, `-X 700/1000` | `-U`                            |
| SAMTOOLS_FILTER       | `-f 0x002` (properly paired)                             | flag req omitted                |
| MACS2_CALLPEAK        | `-f BAMPE`                                               | `-f BAM`                        |
| DEEPTOOLS_BAMCOVERAGE | `--extendReads`                                          | no extension                    |
| SEACR                 | required                                                 | **rejected** — SEACR is PE-only |

**CUT&RUN must be paired-end.** A `cutrun` row with no `fastq_2` (or an SRA
accession that resolves to single-end) is rejected at the peak-calling stage.

### Assay-specific Bowtie2 parameters

| assay   | PE flags                                                                                     | SE flags                                                                 |
| ------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| chipseq | `--very-sensitive --no-unal --no-mixed --no-discordant -X 1000`                              | `--very-sensitive --no-unal`                                             |
| cutrun  | `--local --very-sensitive-local --no-unal --no-mixed --no-discordant --phred33 -I 10 -X 700` | `--local --very-sensitive-local --no-unal --phred33` (rejected by SEACR) |

The CUT&RUN settings follow the Henikoff lab recipe (Skene & Henikoff 2017).

### SRA fetching

SRA rows are downloaded inside the `SRA_FETCH` process (`sra-tools` container):

1. `prefetch --max-size 100g <accession>` — pulls the `.sra` blob to the work dir.
2. `fasterq-dump --split-files --skip-technical --threads N <accession>` —
   converts to FASTQ.
3. Files are renamed to the `sample` ID from the samplesheet and `pigz`-ed.

No NCBI credentials are required for public runs. For very large studies, mount
`~/.ncbi/user-settings.mkfg` into the container to point `prefetch` at a shared
SRA cache.

## Reusing prebuilt indices

```bash
--bowtie2_index /path/to/bowtie2_index/   # skip BOWTIE2_BUILD
```

The directory must be the full Bowtie2 index folder with basename `genome`
(i.e. files like `genome.1.bt2`).

## Tuning parameters

| Param                | Default     | Notes                                                             |
| -------------------- | ----------- | ----------------------------------------------------------------- |
| `--min_mapq`         | `30`        | Minimum MAPQ kept by `SAMTOOLS_FILTER`                            |
| `--keep_dups`        | `false`     | Set `true` to keep duplicate reads (common for CUT&RUN low-input) |
| `--macs_broad`       | `false`     | Pass `--broad` to MACS2 (use for histone marks like H3K27me3)     |
| `--macs_qvalue`      | `0.05`      | MACS2 `-q` threshold                                              |
| `--seacr_threshold`  | `0.01`      | SEACR non-control threshold (fraction of peaks called)            |
| `--seacr_stringency` | `stringent` | SEACR mode: `stringent` or `relaxed`                              |
| `--bigwig_binsize`   | `50`        | deepTools `bamCoverage --binSize`                                 |
| `--bigwig_normalize` | `CPM`       | One of `CPM`, `RPGC`, `BPM`, `RPKM`, `None`                       |
| `--skip_bigwig`      | `false`     | Skip deepTools track generation                                   |
| `--skip_peaks`       | `false`     | Skip MACS2 + SEACR (alignment + QC only)                          |
| `--skip_multiqc`     | `false`     | Skip MultiQC aggregation                                          |

## Outputs

```
results/chipseq/
  fastp/<sample>/                       # trimmed FASTQ, fastp JSON+HTML
  bowtie2/bowtie2_index/                # Bowtie2 index (cached)
  bowtie2/<sample>/                     # sorted.bam, bai, bowtie2.log, flagstat, stats
  markduplicates/<sample>/              # markdup.bam, markdup.metrics.txt
  filtered/<sample>/                    # filt.bam, filt.bam.bai, filt.flagstat
  picard/<sample>/                      # CollectMultipleMetrics outputs (insert size, etc.)
  bigwig/<sample>.bw                    # deepTools bigWig coverage
  macs2/<sample>/                       # narrowPeak / broadPeak / xls / summits.bed
  seacr/<sample>/                       # *.fragments.bedgraph + *.seacr.*.bed
  reference/                            # genome.fa, .fai, chrom.sizes
  multiqc/multiqc_report.html
  pipeline_info/                        # Nextflow execution report/timeline/trace/DAG
```

## Notes

- Bowtie2 index is built once per `--genome_fasta` and cached on disk through
  Nextflow's work cache; use `--bowtie2_index` to reuse across runs.
- This pipeline is intentionally minimal — no `--blacklist` filter, no
  fragment-size partitioning for CUT&RUN nucleosome vs. TF subsets, no
  spike-in normalisation. Add modules under `modules/local/` and include them in
  `workflows/chipseq.nf` when needed.
- A SEACR run without a matched control falls back to the non-control threshold
  mode (`params.seacr_threshold`, default `0.01`).
- `pre-commit` does not lint `.nf`/`.config` files; run `nextflow inspect`
  (`bash pipelines/chipseq/tests/inspect.sh`) locally to validate channel
  topology before pushing.
