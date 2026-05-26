process SEACR_CALLPEAK {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/seacr:1.3--hdfd78af_2'

    publishDir "${params.outdir}/seacr/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(ip_bedgraph), path(ctrl_bedgraph)

    output:
    tuple val(meta), path("${meta.id}.seacr.*.bed"), emit: peaks
    path "versions.yml",                             emit: versions

    script:
    def has_ctrl   = ctrl_bedgraph.name != 'NO_FILE'
    def threshold  = has_ctrl ? "${ctrl_bedgraph}" : "${params.seacr_threshold}"
    def stringency = params.seacr_stringency in ['stringent', 'relaxed'] ? params.seacr_stringency : 'stringent'
    def norm       = has_ctrl ? 'norm' : 'non'
    """
    SEACR_1.3.sh \\
        ${ip_bedgraph} \\
        ${threshold} \\
        ${norm} \\
        ${stringency} \\
        ${meta.id}.seacr

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        seacr: 1.3
    END_VERSIONS
    """
}
