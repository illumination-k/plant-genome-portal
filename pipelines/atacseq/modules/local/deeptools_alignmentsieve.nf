process DEEPTOOLS_ALIGNMENTSIEVE {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/deeptools:3.5.5--pyhdfd78af_0'

    input:
    tuple val(meta), path(bam), path(bai)

    output:
    tuple val(meta), path("${meta.id}.shifted.unsorted.bam"), emit: bam
    path "versions.yml",                                      emit: versions

    script:
    // alignmentSieve --ATACshift applies the canonical Tn5 +4 / -5 correction.
    // Output is not coordinate-sorted; SAMTOOLS_RESORT handles that downstream.
    """
    alignmentSieve \\
        --bam ${bam} \\
        --outFile ${meta.id}.shifted.unsorted.bam \\
        --ATACshift \\
        --numberOfProcessors ${task.cpus}

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        deeptools: \$(alignmentSieve --version 2>&1 | sed 's/^alignmentSieve //')
    END_VERSIONS
    """
}
