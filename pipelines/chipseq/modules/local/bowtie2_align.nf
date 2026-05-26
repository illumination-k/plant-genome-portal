process BOWTIE2_ALIGN {
    tag "${meta.id}"
    label 'process_high'

    container 'quay.io/biocontainers/bowtie2:2.5.4--he20e202_2'

    publishDir "${params.outdir}/bowtie2/${meta.id}", mode: params.publish_dir_mode,
        pattern: '*.log'

    input:
    tuple val(meta), path(reads)
    path index

    output:
    tuple val(meta), path("${meta.id}.sam"), emit: sam
    path "${meta.id}.bowtie2.log",           emit: log
    path "versions.yml",                     emit: versions

    script:
    def reads_arg = meta.single_end ? "-U ${reads}" : "-1 ${reads[0]} -2 ${reads[1]}"

    // Henikoff lab CUT&RUN recipe vs. standard ChIP-seq parameters
    def assay_args
    if (meta.assay == 'cutrun') {
        assay_args = meta.single_end
            ? '--local --very-sensitive-local --no-unal --phred33'
            : '--local --very-sensitive-local --no-unal --no-mixed --no-discordant --phred33 -I 10 -X 700'
    } else {
        assay_args = meta.single_end
            ? '--very-sensitive --no-unal'
            : '--very-sensitive --no-unal --no-mixed --no-discordant -X 1000'
    }

    """
    bowtie2 \\
        -x ${index}/genome \\
        ${reads_arg} \\
        ${assay_args} \\
        --threads ${task.cpus} \\
        -S ${meta.id}.sam \\
        2> ${meta.id}.bowtie2.log

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        bowtie2: \$(bowtie2 --version 2>&1 | head -n1 | sed 's/.* version //')
    END_VERSIONS
    """
}
