#!/usr/bin/env bash
# Syntax + DAG validation for the atacseq pipeline.
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
: >"${TMP}/r.fq.gz"
cat >"${TMP}/s.csv" <<EOF
sample,group,replicate,sra,fastq_1,fastq_2
atac_pe,grp1,1,,${TMP}/r.fq.gz,${TMP}/r.fq.gz
atac_se,grp2,1,,${TMP}/r.fq.gz,
atac_sra,sra,1,SRR1,,
EOF

"${NEXTFLOW}" -quiet inspect "${PIPELINE}" \
	-profile docker \
	--input "${TMP}/s.csv" \
	--genome_fasta "${TMP}/g.fa" \
	--macs_gsize 2.0e4

echo "OK: pipeline inspected"
