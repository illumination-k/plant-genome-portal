process MACS3_CALLPEAK {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/macs3:3.0.4--py312h71493bf_0'

    publishDir "${params.outdir}/macs3/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(bam), path(bai)

    output:
    tuple val(meta), path("${meta.id}_peaks.*"), emit: peaks
    path "${meta.id}.macs3.log",                 emit: log
    path "versions.yml",                         emit: versions

    script:
    if (!params.macs_gsize) {
        error "MACS3_CALLPEAK requires --macs_gsize (e.g. 2.25e8 for Marchantia)"
    }
    def broad_arg = params.macs_broad ? '--broad' : ''
    // Buenrostro 2013 ATAC-seq recipe:
    //   PE: -f BAMPE + --nomodel + --keep-dup all (dups already removed upstream)
    //   SE: -f BAM   + --nomodel --shift -75 --extsize 150 (center read on cut site)
    def fmt_args = meta.single_end
        ? "-f BAM --nomodel --shift ${params.macs_shift} --extsize ${params.macs_extsize}"
        : '-f BAMPE --nomodel'
    """
    macs3 callpeak \\
        -t ${bam} \\
        ${fmt_args} \\
        --keep-dup all \\
        -g ${params.macs_gsize} \\
        -n ${meta.id} \\
        -q ${params.macs_qvalue} \\
        ${broad_arg} \\
        --outdir . \\
        2> ${meta.id}.macs3.log

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        macs3: \$(macs3 --version 2>&1 | sed 's/^macs3 //')
    END_VERSIONS
    """
}
