# ChIP-seq / CUT&RUN pipeline — tests

Syntax / topology tests for `pipelines/chipseq`.

| File         | Purpose                                                                                          |
| ------------ | ------------------------------------------------------------------------------------------------ |
| `inspect.sh` | Runs `nextflow inspect` against dummy inputs to validate syntax and channel topology. No Docker. |
| `lib.sh`     | Shared helper — downloads a pinned Nextflow release into `~/.cache/...` if not already on PATH.  |

## Running locally

```bash
# Syntax + DAG only (no Docker, no container pulls)
bash pipelines/chipseq/tests/inspect.sh
```

## TODO

A deterministic smoke fixture (synthetic IP/control FASTQ with known peak positions)
is not yet implemented. Until then, end-to-end validation is performed on real
ChIP-seq / CUT&RUN datasets. See `pipelines/transcriptome/tests/gen_fixture.py`
for the pattern to follow when one is added.

## CI

`.github/workflows/ci_nextflow.yml` runs `inspect.sh` on every PR or push that
touches `pipelines/chipseq/**`.
