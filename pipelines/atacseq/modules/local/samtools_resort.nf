process SAMTOOLS_RESORT {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/samtools:1.20--h50ea8bc_1'

    publishDir "${params.outdir}/shifted/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(bam)

    output:
    tuple val(meta), path("${meta.id}.shifted.bam"), path("${meta.id}.shifted.bam.bai"), emit: bam
    path "versions.yml",                                                                  emit: versions

    script:
    """
    samtools sort  -@ ${task.cpus} -o ${meta.id}.shifted.bam ${bam}
    samtools index -@ ${task.cpus} ${meta.id}.shifted.bam

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        samtools: \$(samtools --version 2>&1 | head -n1 | sed 's/^samtools //')
    END_VERSIONS
    """
}
