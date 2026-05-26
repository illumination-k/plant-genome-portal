process SAMTOOLS_SORT {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/samtools:1.20--h50ea8bc_1'

    publishDir "${params.outdir}/hisat2/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(sam)

    output:
    tuple val(meta), path("${meta.id}.sorted.bam"), path("${meta.id}.sorted.bam.bai"), emit: bam
    path "${meta.id}.flagstat",                                                        emit: flagstat
    path "${meta.id}.stats",                                                           emit: stats
    path "versions.yml",                                                               emit: versions

    script:
    """
    samtools sort -@ ${task.cpus} -o ${meta.id}.sorted.bam ${sam}
    samtools index -@ ${task.cpus} ${meta.id}.sorted.bam
    samtools flagstat -@ ${task.cpus} ${meta.id}.sorted.bam > ${meta.id}.flagstat
    samtools stats    -@ ${task.cpus} ${meta.id}.sorted.bam > ${meta.id}.stats

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        samtools: \$(samtools --version 2>&1 | head -n1 | sed 's/^samtools //')
    END_VERSIONS
    """
}
