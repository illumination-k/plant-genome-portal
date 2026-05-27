# plant-genome-portal

## Genome MVP

Download and parse MarpolBase MpTak1_v7.1 into a local snapshot. This imports FASTA, GFF3,
functional annotation, and the MarpolBase nomenclature table:

```bash
cargo run -p portal-cli -- import marpolbase-mptak1-v7-1 --out data/marpolbase/MpTak1_v7.1
```

Rebuild only the snapshot from existing input files after changing the snapshot schema:

```bash
cargo run -p portal-cli -- import marpolbase-mptak1-v7-1 --rebuild-snapshot
```

Run the API without any external database or network dependency:

```bash
cargo run -p api -- \
  --bind 127.0.0.1:3000 \
  --snapshot data/marpolbase/MpTak1_v7.1/snapshot.json \
  --fasta data/marpolbase/MpTak1_v7.1/MpTak1_v7.1.fa.gz
```

Run the API and web development servers with file watching:

```bash
mise r dev
```

This starts the API on `127.0.0.1:3000` and the Vite web server with proxying to the API.

Stop both dev servers from another shell:

```bash
mise r stop_dev
```

Useful endpoints:

- `GET /jbrowse/config?baseUrl=http://127.0.0.1:3000`
- `GET /jbrowse/config/GCA_037833805.1?baseUrl=http://127.0.0.1:3000`
- `GET /jbrowse/assemblies/GCA_037833805.1/chrom.sizes`
- `GET /jbrowse/assemblies/GCA_037833805.1/features?refName=chr1&start=0&end=100000`
- `GET /v2/genome/accession/GCA_037833805.1`
- `GET /v2/genome/taxon/3197`
- `GET /v2/gene/search?q=Mp1g00070`
- `GET /v2/gene/id/Mp1g00070`
- `GET /v2/genome/accession/GCA_037833805.1/region/chr1:1-100000/features`
- `GET /sequence/service-info`
- `GET /openapi.json`

Prepare a BLASTN database from the genome FASTA:

```bash
cargo run -p portal-cli -- prepare blastn \
  --fasta data/marpolbase/MpTak1_v7.1/MpTak1_v7.1.fa.gz \
  --out target/blast
```

Then run one worker-side BLASTN homology search and write the domain-normalized result:

```bash
cargo run -p worker -- blastn-once \
  --assembly-accession GCA_037833805.1 \
  --blast-db-prefix target/blast/MpTak1_v7.1 \
  --snapshot data/marpolbase/MpTak1_v7.1/snapshot.json \
  --query ACGTACGTACGT \
  --output target/blast/result.json
```

Worker jobs use the shared `service::WorkerJob<Input>` application envelope. The
worker's current infra adapter supports MessagePack job/result payloads via
`worker blastn-job`, while `blastn-once` is a developer-friendly wrapper that builds
the same typed job from CLI arguments.

Generate the OpenAPI schema and TypeScript client:

```bash
pnpm --dir web run openapi:generate
```

The script writes the API schema to `target/openapi/api.json` via `cargo run -p api -- openapi`, then generates the Hey API client into `web/src/api/client`.

Run OpenAPI-driven property-based tests against the backend:

```bash
mise run pbt:backend
```

The task starts the API with `tests/fixtures/backend-pbt`, then runs Schemathesis against `/openapi.json`. Extra Schemathesis options can be passed through the script:

```bash
bash scripts/pbt/backend-schemathesis.sh --max-examples 200 --continue-on-failure
```

## Comparative and Functional Annotation Pipelines

Two genome-manifest driven Nextflow pipelines are available for adding genomes
after the portal is already running:

```bash
nextflow run pipelines/orthology \
  -profile docker \
  --genomes pipelines/orthology/samplesheets/genomes.example.csv \
  --outdir results/orthology

nextflow run pipelines/interproscan \
  -profile docker \
  --genomes pipelines/interproscan/samplesheets/genomes.example.csv \
  --outdir results/interproscan
```

Append rows to each `genomes.csv` to add species/assemblies. Orthology exports
`results/orthology/portal/orthogroups.tsv`; InterProScan exports
`results/interproscan/portal/functional_annotation.1_line.tsv`.
