# ATAC-seq pipeline

Nextflow DSL2 pipeline that turns raw ATAC-seq reads (local FASTQ or SRA
accession) into:

- `bowtie2/<sample>/*.sorted.bam` — coordinate-sorted, indexed alignments
- `markduplicates/<sample>/` — duplicate-marked BAM + Picard metrics
- `filtered/<sample>/*.filt.bam` — MAPQ + flag + mitochondrial-filtered BAM
- `shifted/<sample>/*.shifted.bam` — Tn5 +4 / -5 shifted BAM (deepTools `alignmentSieve --ATACshift`)
- `bigwig/<sample>.bw` — deepTools `bamCoverage` track (CPM by default)
- `macs3/<sample>/` — MACS3 narrowPeak / broadPeak with ATAC-mode parameters
- `frip/<sample>.frip.tsv` — Fraction of Reads in Peaks (ENCODE QC)
- `homer/<sample>/` — HOMER `findMotifsGenome.pl` known + de novo motif results
- `picard/<sample>/` — `CollectMultipleMetrics` (insert size — captures nucleosome periodicity, alignment summary, ...)
- `multiqc_report.html` — aggregated QC over fastp, Bowtie2, samtools, Picard

ATAC-seq differs from ChIP-seq in a few important ways and this pipeline
handles each explicitly:

1. **No control samples.** ATAC peak calling is single-sample.
2. **Mitochondrial / plastid contamination is huge.** Plant ATAC libraries
   are typically dominated by organelle reads; they are filtered out via
   `--mito_names` before peak calling.
3. **Tn5 binding bias.** Reads are shifted +4 (forward) / -5 (reverse) so the
   alignment represents the actual Tn5 cut site (Buenrostro et al. 2013).
4. **MACS3 in ATAC mode.** `--nomodel --keep-dup all`; PE uses `-f BAMPE`, SE
   centers the read on the cut site with `--shift -75 --extsize 150`.
5. **FRiP** is reported per sample (ENCODE threshold: >= 0.2).

## Layout

```
pipelines/atacseq/
  main.nf                              # entrypoint
  nextflow.config                      # params, profiles, reports
  conf/
    base.config                        # default resources, retries
    test.config                        # minimal test profile
  workflows/
    atacseq.nf                         # wire-up
  modules/local/
    sra_fetch.nf                       # prefetch + fasterq-dump
    fastp.nf                           # QC + adapter trim
    samtools_faidx.nf                  # genome .fai + chrom.sizes
    bowtie2_build.nf                   # Bowtie2 index
    bowtie2_align.nf                   # alignment (-X 2000 for long Tn5 fragments)
    samtools_sort.nf                   # SAM → sorted BAM + flagstat/stats/idxstats
    picard_markduplicates.nf           # MarkDuplicates + metrics
    samtools_filter.nf                 # flag / MAPQ filter + mitochondrial drop
    deeptools_alignmentsieve.nf        # Tn5 +4 / -5 shift
    samtools_resort.nf                 # re-sort + index after shift
    picard_collectmultiplemetrics.nf   # insert size (nucleosome periodicity), etc.
    deeptools_bamcoverage.nf           # bigWig track
    macs3_callpeak.nf                  # ATAC-mode peak calling
    frip.nf                            # Fraction of Reads in Peaks
    homer_findmotifs.nf                # motif analysis on peak BEDs
    multiqc.nf                         # aggregate QC report
  assets/
    multiqc_config.yml
    NO_FILE                            # sentinel for optional inputs
  samplesheets/samplesheet.example.csv
```

## Quick start

```bash
nextflow run pipelines/atacseq \
    -profile docker \
    --input        samplesheet.csv \
    --genome_fasta data/marpolbase/MpTak1_v7.1/MpTak1_v7.1.fa.gz \
    --macs_gsize   2.25e8 \
    --mito_names   'Mp_MT,Mp_PT' \
    --outdir       results/atacseq
```

Use `-profile singularity` on HPC. All processes pin BioContainers images, so
no local installs are required beyond Nextflow (>= 24.04) and a container
runtime.

`--macs_gsize` is required — pass the effective genome size for the organism
(Marchantia `MpTak1_v7.1` ≈ `2.25e8`).

## Samplesheet

CSV with header `sample,group,replicate,sra,fastq_1,fastq_2`.

| column    | required    | notes                                                           |
| --------- | ----------- | --------------------------------------------------------------- |
| sample    | yes         | Sample ID. Used for output filenames and MultiQC grouping.      |
| group     | optional    | Replicate group label. Defaults to `sample` if omitted.         |
| replicate | optional    | Replicate index within `group`. Defaults to `1`.                |
| sra       | optional    | SRA run accession (SRR/ERR/DRR). If set, `fastq_*` are ignored. |
| fastq_1   | when no sra | Path to R1 FASTQ (gz ok).                                       |
| fastq_2   | optional    | Path to R2 for paired-end. Leave blank for single-end.          |

ATAC-seq is almost always paired-end; SE rows are supported but MACS3 will fall
back to `-f BAM --shift -75 --extsize 150` and the insert-size QC plot will be
degenerate.

### Paired-end vs single-end

Layout is detected per sample and propagated as `meta.single_end`:

- **Local FASTQ**: `fastq_2` populated → paired-end; empty → single-end.
- **SRA**: detected after `fasterq-dump --split-files` from the number of FASTQ
  files emitted.

The flag drives PE/SE branches in every downstream process:

| Process         | Paired                                               | Single                                       |
| --------------- | ---------------------------------------------------- | -------------------------------------------- |
| FASTP           | `--in1` / `--in2`, adapter PE detect                 | `--in1` only                                 |
| BOWTIE2_ALIGN   | `-1` / `-2`, `--no-mixed --no-discordant`, `-X 2000` | `-U`                                         |
| SAMTOOLS_FILTER | `-f 0x002` (properly paired)                         | flag req omitted                             |
| MACS3_CALLPEAK  | `-f BAMPE --nomodel --keep-dup all`                  | `-f BAM --nomodel --shift -75 --extsize 150` |

### Mitochondrial / plastid filtering

Plant ATAC libraries are usually 30–80 % organelle reads. `SAMTOOLS_FILTER`
drops any reference whose name matches `--mito_names` (comma-separated, default
`chrM,Mt,MT,Pt,chrPt,chrMt,Mp_MT,Mp_PT`). A per-contig count of dropped reads
is written to `filtered/<sample>/<sample>.mito.txt` for QC.

Override per genome:

```bash
--mito_names 'Mp_MT,Mp_PT'    # Marchantia MpTak1_v7.1
--mito_names 'ChrM,ChrC'      # Arabidopsis TAIR10
--mito_names 'chrM'           # human / mouse
```

### Tn5 shift correction

deepTools `alignmentSieve --ATACshift` applies +4 to forward-strand reads and
-5 to reverse-strand reads so the alignment position represents the Tn5 cut
site. This is the canonical correction from Buenrostro 2013 and matches
ENCODE's ATAC-seq pipeline. Disable with `--atac_shift false` if downstream
tools expect un-shifted reads.

## Reusing prebuilt indices

```bash
--bowtie2_index /path/to/bowtie2_index/   # skip BOWTIE2_BUILD
```

The directory must be the full Bowtie2 index folder with basename `genome`
(i.e. files like `genome.1.bt2`).

## Tuning parameters

| Param                | Default                                 | Notes                                                             |
| -------------------- | --------------------------------------- | ----------------------------------------------------------------- |
| `--mito_names`       | `chrM,Mt,MT,Pt,chrPt,chrMt,Mp_MT,Mp_PT` | Comma-separated organelle contigs to drop                         |
| `--min_mapq`         | `30`                                    | Minimum MAPQ kept by `SAMTOOLS_FILTER`                            |
| `--keep_dups`        | `false`                                 | Keep duplicates after MarkDuplicates (ATAC usually drops them)    |
| `--atac_shift`       | `true`                                  | Apply Tn5 +4 / -5 shift via `alignmentSieve --ATACshift`          |
| `--bowtie2_maxins`   | `2000`                                  | `-X` for Bowtie2 PE alignment                                     |
| `--macs_qvalue`      | `0.05`                                  | MACS3 `-q` threshold                                              |
| `--macs_broad`       | `false`                                 | Pass `--broad` to MACS3 (chromatin domains; usually off for ATAC) |
| `--macs_shift`       | `-75`                                   | MACS3 `--shift` for SE only                                       |
| `--macs_extsize`     | `150`                                   | MACS3 `--extsize` for SE only                                     |
| `--homer_size`       | `200`                                   | HOMER window around peak center (bp)                              |
| `--homer_mset`       | `null`                                  | HOMER motif set: `plants` / `vertebrates` / ... (default: auto)   |
| `--homer_extra_args` | `-mask`                                 | Extra `findMotifsGenome.pl` flags                                 |
| `--bigwig_binsize`   | `50`                                    | deepTools `bamCoverage --binSize`                                 |
| `--bigwig_normalize` | `CPM`                                   | One of `CPM`, `RPGC`, `BPM`, `RPKM`, `None`                       |
| `--skip_bigwig`      | `false`                                 | Skip deepTools track generation                                   |
| `--skip_peaks`       | `false`                                 | Skip MACS3 (alignment + QC only). Also suppresses FRiP and HOMER. |
| `--skip_frip`        | `false`                                 | Skip FRiP calculation                                             |
| `--skip_motifs`      | `false`                                 | Skip HOMER motif analysis                                         |
| `--skip_multiqc`     | `false`                                 | Skip MultiQC aggregation                                          |

## Outputs

```
results/atacseq/
  fastp/<sample>/                       # trimmed FASTQ, fastp JSON+HTML
  bowtie2/bowtie2_index/                # Bowtie2 index (cached)
  bowtie2/<sample>/                     # sorted.bam, bai, bowtie2.log, flagstat, stats, idxstats
  markduplicates/<sample>/              # markdup.bam, markdup.metrics.txt
  filtered/<sample>/                    # filt.bam, filt.bam.bai, filt.flagstat, mito.txt
  shifted/<sample>/                     # shifted.bam, shifted.bam.bai (Tn5 +4/-5)
  picard/<sample>/                      # CollectMultipleMetrics (insert size → nucleosome ladder)
  bigwig/<sample>.bw                    # deepTools bigWig coverage
  macs3/<sample>/                       # narrowPeak / broadPeak / xls / summits.bed
  frip/<sample>.frip.tsv                # FRiP score
  homer/<sample>/<sample>_homer/        # knownResults.html + homerResults.html (de novo)
  reference/                            # genome.fa, .fai, chrom.sizes
  multiqc/multiqc_report.html
  pipeline_info/                        # Nextflow execution report/timeline/trace/DAG
```

## Notes

- The insert-size distribution from `picard CollectMultipleMetrics` is the
  primary periodicity QC for ATAC — a high-quality library shows a nucleosome
  ladder (peaks at ~50, 200, 400 bp).
- This pipeline is intentionally minimal — no blacklist filter, no TSS
  enrichment plot, no fragment-size partitioning into nucleosome-free /
  mono-/di-nucleosome subsets, no spike-in normalisation. Add modules under
  `modules/local/` and include them in `workflows/atacseq.nf` when needed.
- For TSS enrichment, the typical add-on is deepTools `computeMatrix
  reference-point -S sample.bw -R tss.bed` + `plotProfile`. It needs a TSS BED
  derived from the GFF, so it's deferred until the gene model is wired in.
- `pre-commit` does not lint `.nf`/`.config` files; run `nextflow inspect`
  (`bash pipelines/atacseq/tests/inspect.sh`) locally to validate channel
  topology before pushing.

### Why MACS3 and not Genrich / HMMRATAC?

- **MACS3** — most-cited ATAC peak caller; well-supported in BioContainers;
  consistent with the chipseq pipeline already in this repo. Default here.
- **Genrich** (Gaspar) — handles PE fragments natively, has an ATAC mode that
  treats each fragment as an interval. Worth swapping in if you observe
  substantially different peak boundaries than expected.
- **HMMRATAC** (Tarbell & Liu 2019) — purpose-built for ATAC, segments
  nucleosome-free vs nucleosome-bound regions. Heavier dependency, fewer
  downstream tools support its output. Not part of the default pipeline.
