process SAMTOOLS_NSORT {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/samtools:1.20--h50ea8bc_1'

    input:
    tuple val(meta), path(bam), path(bai)

    output:
    tuple val(meta), path("${meta.id}.nsorted.bam"), emit: bam
    path "versions.yml",                             emit: versions

    script:
    """
    samtools sort -n -@ ${task.cpus} -o ${meta.id}.nsorted.bam ${bam}

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        samtools: \$(samtools --version 2>&1 | head -n1 | sed 's/^samtools //')
    END_VERSIONS
    """
}
