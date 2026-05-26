process SALMON_QUANT {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/salmon:1.10.3--h6dccd9a_2'

    publishDir "${params.outdir}/salmon/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(reads)
    path index

    output:
    tuple val(meta), path("quant"), emit: results
    path "versions.yml",            emit: versions

    script:
    def lib_type = 'A'
    if (meta.strandedness == 'forward')    lib_type = meta.single_end ? 'SF' : 'ISF'
    if (meta.strandedness == 'reverse')    lib_type = meta.single_end ? 'SR' : 'ISR'
    if (meta.strandedness == 'unstranded') lib_type = meta.single_end ? 'U'  : 'IU'

    def reads_arg = meta.single_end ? "-r ${reads}" : "-1 ${reads[0]} -2 ${reads[1]}"

    """
    salmon quant \\
        --threads ${task.cpus} \\
        --libType ${lib_type} \\
        -i ${index} \\
        ${reads_arg} \\
        --validateMappings \\
        --gcBias \\
        --seqBias \\
        -o quant

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        salmon: \$(salmon --version 2>&1 | sed 's/^salmon //')
    END_VERSIONS
    """
}
