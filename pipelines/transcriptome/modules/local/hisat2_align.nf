process HISAT2_ALIGN {
    tag "${meta.id}"
    label 'process_high'

    container 'quay.io/biocontainers/hisat2:2.2.1--h87f3376_5'

    publishDir "${params.outdir}/hisat2/${meta.id}", mode: params.publish_dir_mode,
        pattern: '*.{log,sam}'

    input:
    tuple val(meta), path(reads)
    path index

    output:
    tuple val(meta), path("${meta.id}.sam"),         emit: sam
    path "${meta.id}.hisat2.summary.log",            emit: summary
    path "versions.yml",                             emit: versions

    script:
    def reads_arg = meta.single_end ? "-U ${reads}" : "-1 ${reads[0]} -2 ${reads[1]}"

    def strand_arg = ''
    if (meta.strandedness == 'forward') strand_arg = meta.single_end ? '--rna-strandness F'  : '--rna-strandness FR'
    if (meta.strandedness == 'reverse') strand_arg = meta.single_end ? '--rna-strandness R'  : '--rna-strandness RF'

    """
    hisat2 \\
        -x ${index}/genome \\
        ${reads_arg} \\
        ${strand_arg} \\
        --threads ${task.cpus} \\
        --no-unal \\
        --new-summary \\
        --summary-file ${meta.id}.hisat2.summary.log \\
        -S ${meta.id}.sam

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        hisat2: \$(hisat2 --version 2>&1 | head -n1 | sed 's/.*hisat2-align-s version //')
    END_VERSIONS
    """
}
