# Orthology pipeline

Nextflow DSL2 pipeline for adding comparative genomes to Plant Genome Portal.
It normalizes protein FASTA headers, runs OrthoFinder across all genomes in a
manifest, and exports the portal orthogroup TSV consumed by:

```bash
portal-cli import marpolbase-mptak1-v7-1 --orthogroups results/orthology/portal/orthogroups.tsv
```

## Quick Start

```bash
nextflow run pipelines/orthology \
    -profile docker \
    --genomes pipelines/orthology/samplesheets/genomes.example.csv \
    --outdir results/orthology
```

Add a genome by appending one row to `genomes.csv`. OrthoFinder is a
cohort-wide analysis, so adding a genome reruns the OrthoFinder step against
the full manifest; `-resume` still reuses unchanged prepare tasks.

Override containers when your runtime needs a local mirror:

```bash
--orthofinder_container quay.io/biocontainers/orthofinder:2.5.5--hdfd78af_1
--python_container python:3.12-slim
```

## Genome Manifest

CSV header:

```text
genome_id,tax_id,scientific_name,assembly_accession,protein_fasta,gene_metadata_tsv
```

| column               | required | notes                                                                                   |
| -------------------- | -------- | --------------------------------------------------------------------------------------- |
| `genome_id`          | yes      | Stable short ID. This becomes the OrthoFinder species column.                           |
| `tax_id`             | yes      | NCBI taxonomy ID.                                                                       |
| `scientific_name`    | yes      | Display species name.                                                                   |
| `assembly_accession` | yes      | Portal/current assembly accession where available.                                      |
| `protein_fasta`      | yes      | Protein FASTA, optionally gzipped. FASTA IDs become OrthoFinder sequence IDs.           |
| `gene_metadata_tsv`  | optional | TSV mapping FASTA IDs to portal genes. Leave blank when FASTA IDs are already gene IDs. |

`gene_metadata_tsv` accepts `protein_id`, `transcript_id`, or `sequence_id` as
the FASTA ID column, plus required `gene_id` and optional `symbol`:

```text
protein_id    gene_id    symbol
Mp1g00070.1   Mp1g00070  BARD1
```

For the current portal assembly, the exported `gene_id` values must match genes
in the snapshot, because snapshot build validates same-assembly orthogroup
members.

## Outputs

```text
results/orthology/
  prepared/                  # normalized per-genome protein FASTA + gene metadata
  orthofinder/               # raw OrthoFinder output
  portal/orthogroups.tsv     # portal snapshot input
  pipeline_info/
```

`portal/orthogroups.tsv` uses these tab-separated columns:

```text
orthogroup_id    gene_id    tax_id    scientific_name    assembly_accession    symbol
```
