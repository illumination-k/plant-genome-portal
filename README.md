# plant-genome-portal

## Genome MVP

Download and parse MarpolBase MpTak1_v7.1 into a local snapshot:

```bash
cargo run -p portal-cli -- import marpolbase-mptak1-v7-1 --out data/marpolbase/MpTak1_v7.1
```

Run the API without any external database or network dependency:

```bash
cargo run -p api -- \
  --bind 127.0.0.1:3000 \
  --snapshot data/marpolbase/MpTak1_v7.1/snapshot.json \
  --fasta data/marpolbase/MpTak1_v7.1/MpTak1_v7.1.fa.gz
```

Useful endpoints:

- `GET /v2/genome/accession/GCA_037833805.1`
- `GET /v2/genome/taxon/3197`
- `GET /v2/gene/search?q=Mp1g00070`
- `GET /v2/gene/id/Mp1g00070`
- `GET /v2/genome/accession/GCA_037833805.1/region/chr1:1-100000/features`
- `GET /sequence/service-info`
- `GET /openapi.json`
