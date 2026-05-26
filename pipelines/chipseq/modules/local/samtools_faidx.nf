process SAMTOOLS_FAIDX {
    tag "${genome.simpleName}"
    label 'process_low'

    container 'quay.io/biocontainers/samtools:1.20--h50ea8bc_1'

    publishDir "${params.outdir}/reference", mode: params.publish_dir_mode

    input:
    path genome

    output:
    path "genome.fa",         emit: fasta
    path "genome.fa.fai",     emit: fai
    path "chrom.sizes",       emit: sizes
    path "versions.yml",      emit: versions

    script:
    """
    set -euo pipefail

    if [[ "${genome}" == *.gz ]]; then
        zcat ${genome} > genome.fa
    else
        cp ${genome} genome.fa
    fi

    samtools faidx genome.fa
    cut -f1,2 genome.fa.fai > chrom.sizes

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        samtools: \$(samtools --version 2>&1 | head -n1 | sed 's/^samtools //')
    END_VERSIONS
    """
}
