#!/usr/bin/env bash
# Syntax + DAG validation for the transcriptome pipeline.
#
# Runs `nextflow inspect` against dummy inputs so every process resolves and
# every channel topology is wired correctly. Does NOT pull containers or
# execute anything.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PIPELINE="$(cd "${HERE}/.." && pwd)"

# shellcheck source=lib.sh
# shellcheck disable=SC1091
. "${HERE}/lib.sh"
ensure_nextflow

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

: >"${TMP}/g.fa"
: >"${TMP}/x.gtf"
: >"${TMP}/t.fa"
: >"${TMP}/r.fq.gz"
cat >"${TMP}/s.csv" <<EOF
sample,sra,fastq_1,fastq_2,strandedness
sra_pe,SRR1,,,auto
local_se,,${TMP}/r.fq.gz,,auto
local_pe,,${TMP}/r.fq.gz,${TMP}/r.fq.gz,reverse
EOF

"${NEXTFLOW}" -quiet inspect "${PIPELINE}" \
	-profile docker \
	--input "${TMP}/s.csv" \
	--genome_fasta "${TMP}/g.fa" \
	--gtf "${TMP}/x.gtf" \
	--transcripts_fasta "${TMP}/t.fa"

echo "OK: pipeline inspected"
