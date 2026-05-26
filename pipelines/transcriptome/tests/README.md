# Transcriptome pipeline — tests

Smoke + syntax tests for `pipelines/transcriptome`.

| File             | Purpose                                                                                          |
| ---------------- | ------------------------------------------------------------------------------------------------ |
| `gen_fixture.py` | Builds a deterministic synthetic fixture (2-chr genome, 2 transcripts, paired + single FASTQ).   |
| `inspect.sh`     | Runs `nextflow inspect` against dummy inputs to validate syntax and channel topology. No Docker. |
| `run_smoke.sh`   | Generates the fixture and runs the full pipeline under `-profile docker`, then asserts outputs.  |
| `ci.config`      | Resource caps (2 CPU / 4 GB) so the smoke run fits on a GitHub-hosted runner.                    |
| `lib.sh`         | Shared helper — downloads a pinned Nextflow release into `~/.cache/...` if not already on PATH.  |

## Running locally

```bash
# Syntax + DAG only (no Docker, no container pulls)
bash pipelines/transcriptome/tests/inspect.sh

# Full end-to-end smoke test (requires a working Docker daemon)
bash pipelines/transcriptome/tests/run_smoke.sh
```

`run_smoke.sh` writes intermediate files to `pipelines/transcriptome/tests/_work/`
and final outputs to `pipelines/transcriptome/tests/_results/`. Both paths are
gitignored.

## Assertions

The fixture is seeded so Salmon NumReads are exactly reproducible:

| sample  | layout     | t1 NumReads | t2 NumReads |
| ------- | ---------- | ----------- | ----------- |
| sampleA | paired-end | 120         | 100         |
| sampleB | single-end | 80          | 60          |

HISAT2 alignment rate must be 100 % for both samples. MultiQC report, sorted
BAM + index, Qualimap RNA-seq report, and fastp JSON are checked for existence.

## CI

`.github/workflows/ci_nextflow.yml` runs two jobs on every PR or push that
touches `pipelines/transcriptome/**`:

- **inspect** — `inspect.sh` (lightweight, ~30 s)
- **smoke** — `run_smoke.sh` end-to-end on a `ubuntu-latest` runner (~6-8 min;
  most of that is container pulls on cold cache)
