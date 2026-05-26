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
    def pe_arg = meta.single_end ? '' : '--extendReads'
    """
    bamCoverage \\
        --bam ${bam} \\
        -o ${meta.id}.bw \\
        --binSize ${params.bigwig_binsize} \\
        ${norm_arg} \\
        ${pe_arg} \\
        -p ${task.cpus}

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        deeptools: \$(bamCoverage --version 2>&1 | sed 's/^bamCoverage //')
    END_VERSIONS
    """
}
