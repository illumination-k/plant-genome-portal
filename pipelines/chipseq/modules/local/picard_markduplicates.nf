process PICARD_MARKDUPLICATES {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/picard:3.2.0--hdfd78af_1'

    publishDir "${params.outdir}/markduplicates/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(bam), path(bai)

    output:
    tuple val(meta), path("${meta.id}.markdup.bam"), path("${meta.id}.markdup.bam.bai"), emit: bam
    path "${meta.id}.markdup.metrics.txt",                                               emit: metrics
    path "versions.yml",                                                                 emit: versions

    script:
    def mem_g = (task.memory.giga as int)
    """
    set -euo pipefail

    picard -Xmx${mem_g}g MarkDuplicates \\
        I=${bam} \\
        O=${meta.id}.markdup.bam \\
        M=${meta.id}.markdup.metrics.txt \\
        REMOVE_DUPLICATES=false \\
        ASSUME_SORT_ORDER=coordinate \\
        VALIDATION_STRINGENCY=LENIENT

    # Picard's BAI is .bai (not .bam.bai); rename for downstream
    if [[ -f ${meta.id}.markdup.bai ]]; then
        mv ${meta.id}.markdup.bai ${meta.id}.markdup.bam.bai
    else
        picard -Xmx${mem_g}g BuildBamIndex I=${meta.id}.markdup.bam
        mv ${meta.id}.markdup.bai ${meta.id}.markdup.bam.bai
    fi

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        picard: \$(picard MarkDuplicates --version 2>&1 | sed 's/^Version://')
    END_VERSIONS
    """
}
