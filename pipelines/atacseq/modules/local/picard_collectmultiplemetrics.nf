process PICARD_COLLECTMULTIPLEMETRICS {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/picard:3.2.0--hdfd78af_1'

    publishDir "${params.outdir}/picard/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(bam), path(bai)
    path genome

    output:
    tuple val(meta), path("${meta.id}.CollectMultipleMetrics.*"), emit: metrics
    path "versions.yml",                                          emit: versions

    script:
    def mem_g = (task.memory.giga as int)
    """
    set -euo pipefail

    if [[ "${genome}" == *.gz ]]; then
        zcat ${genome} > genome.fa
    else
        cp ${genome} genome.fa
    fi

    picard -Xmx${mem_g}g CollectMultipleMetrics \\
        I=${bam} \\
        O=${meta.id}.CollectMultipleMetrics \\
        R=genome.fa \\
        VALIDATION_STRINGENCY=LENIENT

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        picard: \$(picard CollectMultipleMetrics --version 2>&1 | sed 's/^Version://')
    END_VERSIONS
    """
}
