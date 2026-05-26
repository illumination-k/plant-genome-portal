process MACS2_CALLPEAK {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/macs2:2.2.9.1--py39hf95cd2a_0'

    publishDir "${params.outdir}/macs2/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(ip_bam), path(ip_bai), path(ctrl_bam), path(ctrl_bai)

    output:
    tuple val(meta), path("${meta.id}_peaks.*"), emit: peaks
    path "${meta.id}.macs2.log",                 emit: log
    path "versions.yml",                         emit: versions

    script:
    if (!params.macs_gsize) {
        error "MACS2_CALLPEAK requires --macs_gsize (e.g. 2.25e8 for Marchantia)"
    }
    def fmt       = meta.single_end ? 'BAM' : 'BAMPE'
    def broad_arg = params.macs_broad ? '--broad' : ''
    def ctrl_arg  = (ctrl_bam.name != 'NO_FILE') ? "-c ${ctrl_bam}" : ''
    """
    macs2 callpeak \\
        -t ${ip_bam} \\
        ${ctrl_arg} \\
        -f ${fmt} \\
        -g ${params.macs_gsize} \\
        -n ${meta.id} \\
        -q ${params.macs_qvalue} \\
        ${broad_arg} \\
        --outdir . \\
        2> ${meta.id}.macs2.log

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        macs2: \$(macs2 --version 2>&1 | sed 's/^macs2 //')
    END_VERSIONS
    """
}
