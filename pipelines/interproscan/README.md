# InterProScan Pipeline

Nextflow DSL2 pipeline for adding functional annotations to Plant Genome Portal.
It normalizes each genome's protein FASTA, runs InterProScan per genome, and
exports MarpolBase-style one-line annotation TSV files.

## Quick Start

```bash
nextflow run pipelines/interproscan \
    -profile docker \
    --genomes pipelines/interproscan/samplesheets/genomes.example.csv \
    --outdir results/interproscan
```

Add a genome by appending one row to `genomes.csv`. Each genome runs as an
independent InterProScan task, so `-resume` can reuse existing per-genome
results when the manifest grows.

InterProScan is large and site-specific on many clusters. Override the container
or pass application filters as needed:

```bash
--interproscan_container quay.io/biocontainers/interproscan:5.73_104.0--hec16e2b_0
--applications Pfam,NCBIfam,Gene3D,SUPERFAMILY
--python_container python:3.12-slim
```

## Genome Manifest

CSV header:

```text
genome_id,tax_id,scientific_name,assembly_accession,protein_fasta,gene_metadata_tsv
```

| column               | required | notes                                                                              |
| -------------------- | -------- | ---------------------------------------------------------------------------------- |
| `genome_id`          | yes      | Stable short ID. Used for per-genome output folders.                               |
| `tax_id`             | yes      | NCBI taxonomy ID. Kept for consistency with orthology manifests.                   |
| `scientific_name`    | yes      | Display species name.                                                              |
| `assembly_accession` | yes      | Portal/current assembly accession where available.                                 |
| `protein_fasta`      | yes      | Protein FASTA, optionally gzipped. FASTA IDs become functional annotation row IDs. |
| `gene_metadata_tsv`  | optional | TSV mapping FASTA IDs to genes for prepared metadata and downstream joins.         |

`gene_metadata_tsv` accepts `protein_id`, `transcript_id`, or `sequence_id` as
the FASTA ID column, plus required `gene_id` and optional `symbol`.

For importing into the current snapshot, FASTA IDs should match portal
transcript IDs because `functional_annotation.1_line.tsv` is applied at
transcript level.

## Outputs

```text
results/interproscan/
  prepared/                                      # normalized FASTA per genome
  interproscan/<genome_id>/                      # raw InterProScan TSV/GFF3
  portal/<genome_id>/*.func_annotation.1_line.tsv
  portal/functional_annotation.1_line.tsv        # combined output
  pipeline_info/
```

The portal TSV has two tab-separated columns:

```text
protein_or_transcript_id    annotation; annotation; ...
```

The converter currently emits GO, InterPro, Pfam, NCBIfam, and KOG-style entries
when those are present in the InterProScan TSV.
