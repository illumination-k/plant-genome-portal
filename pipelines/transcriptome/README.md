# Transcriptome pipeline

Nextflow DSL2 pipeline that turns raw RNA-seq reads (local FASTQ or SRA accession) into:

- `salmon quant/` — transcript-level abundance (TPM, NumReads) per sample
- `hisat2/*.sorted.bam` — coordinate-sorted, indexed BAMs
- `qualimap/` — Qualimap `rnaseq` BAM-QC reports
- `multiqc_report.html` — aggregated QC over fastp, HISAT2, samtools, Qualimap, Salmon

Designed to feed the `expression-store` snapshot (see `docs/current_plan.md` P4).

## Layout

```
pipelines/transcriptome/
  main.nf                       # entrypoint
  nextflow.config               # params, profiles, reports
  conf/
    base.config                 # default resources, retries
    test.config                 # minimal test profile
  workflows/
    transcriptome.nf            # wire-up
  modules/local/
    sra_fetch.nf                # prefetch + fasterq-dump
    fastp.nf                    # QC + adapter trim
    salmon_index.nf             # decoy-aware Salmon index
    salmon_quant.nf             # transcript quantification
    hisat2_build.nf             # HISAT2 index (with splice sites + exons)
    hisat2_align.nf             # spliced alignment → SAM
    samtools_sort.nf            # SAM → sorted BAM + flagstat + stats
    qualimap_rnaseq.nf          # BAM QC (Qualimap rnaseq mode)
    multiqc.nf                  # aggregate QC report
  assets/multiqc_config.yml
  samplesheets/samplesheet.example.csv
```

## Quick start

```bash
nextflow run pipelines/transcriptome \
    -profile docker \
    --input             samplesheet.csv \
    --genome_fasta      data/marpolbase/MpTak1_v7.1/MpTak1_v7.1.fa.gz \
    --gtf               data/marpolbase/MpTak1_v7.1/MpTak1_v7.1.gtf.gz \
    --transcripts_fasta data/marpolbase/MpTak1_v7.1/MpTak1_v7.1.transcripts.fa.gz \
    --outdir            results/transcriptome
```

Use `-profile singularity` on HPC. All processes pin BioContainers images, so no
local installs are required beyond Nextflow (>= 24.04) and a container runtime.

## Samplesheet

CSV with header `sample,sra,fastq_1,fastq_2,strandedness`.

| column       | required    | notes                                                           |
| ------------ | ----------- | --------------------------------------------------------------- |
| sample       | yes         | Sample ID. Used for output filenames and MultiQC grouping.      |
| sra          | optional    | SRA run accession (SRR/ERR/DRR). If set, fastq\_\* are ignored. |
| fastq_1      | when no sra | Path to R1 FASTQ (gz ok).                                       |
| fastq_2      | optional    | Path to R2 for paired-end. Leave blank for single-end.          |
| strandedness | optional    | `auto` (default) / `forward` / `reverse` / `unstranded`.        |

`auto` lets Salmon infer the library type and disables HISAT2 strand flags.
For Qualimap RNA-seq QC, only explicit `forward`/`reverse` switch on strand mode.

### Paired-end vs single-end

Layout is detected per sample and propagated as `meta.single_end`:

- **Local FASTQ**: `fastq_2` populated → paired-end; empty → single-end.
- **SRA**: detected after `fasterq-dump --split-files` from the number of FASTQ
  files emitted (`_1.fastq` + `_2.fastq` = paired; single `<accession>.fastq` =
  single).

The flag drives PE/SE branches in every downstream process:

| Process         | Paired                              | Single                       |
| --------------- | ----------------------------------- | ---------------------------- |
| FASTP           | `--in1`/`--in2`, adapter PE detect  | `--in1` only                 |
| SALMON_QUANT    | libType `ISF`/`ISR`/`IU`/`A`        | libType `SF`/`SR`/`U`/`A`    |
| HISAT2_ALIGN    | `-1`/`-2`, `--rna-strandness FR/RF` | `-U`, `--rna-strandness F/R` |
| QUALIMAP_RNASEQ | `-pe`                               | no `-pe`                     |

A samplesheet may mix PE and SE rows freely; both are validated end-to-end
against a synthetic fixture in `pipelines/transcriptome` development.

### SRA fetching

SRA rows are downloaded inside the `SRA_FETCH` process (`sra-tools` container):

1. `prefetch --max-size 100g <accession>` — pulls the `.sra` blob to the work dir.
2. `fasterq-dump --split-files --skip-technical --threads N <accession>` —
   converts to FASTQ; produces `_1.fastq`/`_2.fastq` for paired libraries or
   `<accession>.fastq` for single.
3. Files are renamed to the `sample` ID from the samplesheet and `pigz`-ed.

No NCBI credentials are required for public runs. For very large studies, mount
`~/.ncbi/user-settings.mkfg` into the container to point `prefetch` at a shared
SRA cache.

## Reusing prebuilt indices

```bash
--salmon_index  /path/to/salmon_index/   # skip SALMON_INDEX
--hisat2_index  /path/to/hisat2_index/   # skip HISAT2_BUILD
```

The directory must be the full Salmon/HISAT2 index folder. The HISAT2 index must
have been built with basename `genome` (i.e. files like `genome.1.ht2`).

## Outputs

```
results/transcriptome/
  fastp/<sample>/                       # trimmed FASTQ, fastp JSON+HTML
  salmon/salmon_index/                  # decoy-aware index (cached)
  salmon/<sample>/quant/                # quant.sf, lib_format_counts.json, ...
  hisat2/hisat2_index/                  # HISAT2 index (cached)
  hisat2/<sample>/                      # sorted.bam, bai, flagstat, stats, summary.log
  qualimap/<sample>/<sample>_rnaseq_qc/ # Qualimap rnaseq output
  multiqc/multiqc_report.html
  pipeline_info/                        # Nextflow execution report/timeline/trace/DAG
```

The downstream `portal-cli import` step consumes `salmon/<sample>/quant/quant.sf`
for TPM + raw count, aggregated to gene level via tx2gene (added in P4 — not
part of this pipeline).

## Notes

- Salmon index is built once per `(transcripts_fasta, genome_fasta)` pair and
  cached on disk through Nextflow's work cache; use `--salmon_index` to reuse
  across runs.
- The pipeline is intentionally minimal — no STAR, no rRNA filter, no UMI
  handling. Add modules under `modules/local/` and include them in
  `workflows/transcriptome.nf` when needed.
- `pre-commit` does not lint `.nf`/`.config` files; run `nextflow run … -preview`
  locally to validate channel topology before pushing.
