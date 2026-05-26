process DEEPTOOLS_BAMCOVERAGE {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/deeptools:3.5.5--pyhdfd78af_0'

    publishDir "${params.outdir}/bigwig", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(bam), path(bai)

    output:
    tuple val(meta), path("${meta.id}.bw"), emit: bigwig
    path "versions.yml",                    emit: versions

    script:
    def norm = (params.bigwig_normalize ?: 'CPM')
    def norm_arg = norm == 'None' ? '' : "--normalizeUsing ${norm}"
    // For ATAC: do NOT --extendReads when the input is already Tn5-shifted;
    // shifted reads represent open-chromatin cut points and should be plotted
    // as-is. bigWig is built from the per-base read pile, which is what we want
    // for visualizing accessibility footprints.
    """
    bamCoverage \\
        --bam ${bam} \\
        -o ${meta.id}.bw \\
        --binSize ${params.bigwig_binsize} \\
        ${norm_arg} \\
        -p ${task.cpus}

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        deeptools: \$(bamCoverage --version 2>&1 | sed 's/^bamCoverage //')
    END_VERSIONS
    """
}
