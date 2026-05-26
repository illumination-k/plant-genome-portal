#!/usr/bin/env bash
# Syntax + DAG validation for the chipseq pipeline.
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
sample,group,replicate,control,assay,sra,fastq_1,fastq_2
ip_pe_with_ctrl,grp1,1,ctrl_pe,chipseq,,${TMP}/r.fq.gz,${TMP}/r.fq.gz
ctrl_pe,ctrl,1,,chipseq,,${TMP}/r.fq.gz,${TMP}/r.fq.gz
ip_se_no_ctrl,grp2,1,,chipseq,,${TMP}/r.fq.gz,
cnr_pe_with_ctrl,cnr1,1,ctrl_pe,cutrun,,${TMP}/r.fq.gz,${TMP}/r.fq.gz
cnr_pe_no_ctrl,cnr2,1,,cutrun,,${TMP}/r.fq.gz,${TMP}/r.fq.gz
sra_pe,sra,1,,chipseq,SRR1,,
EOF

"${NEXTFLOW}" -quiet inspect "${PIPELINE}" \
	-profile docker \
	--input "${TMP}/s.csv" \
	--genome_fasta "${TMP}/g.fa" \
	--macs_gsize 2.0e4

echo "OK: pipeline inspected"
