#!/usr/bin/env bash
# End-to-end smoke test for the transcriptome pipeline.
#
# Generates a deterministic synthetic fixture and drives the full pipeline
# under -profile docker, then asserts that Salmon NumReads exactly match
# the ground-truth read counts. Requires a working Docker daemon.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PIPELINE="$(cd "${HERE}/.." && pwd)"
WORK="${WORK:-${HERE}/_work}"
RESULTS="${RESULTS:-${HERE}/_results}"

# shellcheck source=lib.sh
# shellcheck disable=SC1091
. "${HERE}/lib.sh"
ensure_nextflow

if ! docker version >/dev/null 2>&1; then
	echo "ERROR: docker daemon is not reachable" >&2
	exit 1
fi

rm -rf "${WORK}" "${RESULTS}"
mkdir -p "${WORK}/fixtures"

python3 "${HERE}/gen_fixture.py" "${WORK}/fixtures"

cat >"${WORK}/samplesheet.csv" <<EOF
sample,sra,fastq_1,fastq_2,strandedness
sampleA,,${WORK}/fixtures/sampleA_1.fastq.gz,${WORK}/fixtures/sampleA_2.fastq.gz,unstranded
sampleB,,${WORK}/fixtures/sampleB_se.fastq.gz,,unstranded
EOF

"${NEXTFLOW}" run "${PIPELINE}" \
	-profile docker \
	-c "${HERE}/ci.config" \
	-work-dir "${WORK}/nf-work" \
	--input "${WORK}/samplesheet.csv" \
	--genome_fasta "${WORK}/fixtures/genome.fa" \
	--gtf "${WORK}/fixtures/annotation.gtf" \
	--transcripts_fasta "${WORK}/fixtures/transcripts.fa" \
	--outdir "${RESULTS}" \
	-ansi-log false

# ----- assertions -----
fail() {
	echo "FAIL: $*" >&2
	exit 1
}

assert_count() {
	local f=$1 tx=$2 expected=$3 got
	[ -f "${f}" ] || fail "missing ${f}"
	got=$(awk -v t="${tx}" '$1==t {printf "%d", $5; exit}' "${f}")
	if [ "${got}" != "${expected}" ]; then
		fail "${f} ${tx}: expected ${expected}, got ${got}"
	fi
	echo "OK: $(basename "$(dirname "$(dirname "${f}")")")/${tx} NumReads=${got}"
}

assert_file() {
	[ -e "$1" ] || fail "missing $1"
	echo "OK: exists $(realpath --relative-to="${RESULTS}" "$1")"
}

# Salmon quant — deterministic NumReads from the seeded fixture
assert_count "${RESULTS}/salmon/sampleA/quant/quant.sf" t1 120
assert_count "${RESULTS}/salmon/sampleA/quant/quant.sf" t2 100
assert_count "${RESULTS}/salmon/sampleB/quant/quant.sf" t1 80
assert_count "${RESULTS}/salmon/sampleB/quant/quant.sf" t2 60

# Required QC artifacts (both PE and SE)
assert_file "${RESULTS}/fastp/sampleA/sampleA.fastp.json"
assert_file "${RESULTS}/fastp/sampleB/sampleB.fastp.json"
assert_file "${RESULTS}/hisat2/sampleA/sampleA.sorted.bam"
assert_file "${RESULTS}/hisat2/sampleA/sampleA.sorted.bam.bai"
assert_file "${RESULTS}/hisat2/sampleB/sampleB.sorted.bam"
assert_file "${RESULTS}/qualimap/sampleA/sampleA_rnaseq_qc/rnaseq_qc_results.txt"
assert_file "${RESULTS}/qualimap/sampleB/sampleB_rnaseq_qc/rnaseq_qc_results.txt"
assert_file "${RESULTS}/multiqc/multiqc_report.html"

# 100% alignment rate from HISAT2 — confirms PE/SE branch wiring
grep -q 'Aligned concordantly 1 time: 220 (100.00%)' \
	"${RESULTS}/hisat2/sampleA/sampleA.hisat2.summary.log" ||
	fail "sampleA: HISAT2 PE alignment rate did not hit 100%"
grep -q 'Aligned 1 time: 140 (100.00%)' \
	"${RESULTS}/hisat2/sampleB/sampleB.hisat2.summary.log" ||
	fail "sampleB: HISAT2 SE alignment rate did not hit 100%"

echo "Smoke test PASSED"
