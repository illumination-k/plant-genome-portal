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
    def maxins    = params.bowtie2_maxins ?: 2000

    // ATAC-seq alignment: very-sensitive + extended fragment size cap so the
    // Tn5 long-fragment tail (mono-/di-nucleosome) is retained.
    def assay_args = meta.single_end
        ? '--very-sensitive --no-unal'
        : "--very-sensitive --no-unal --no-mixed --no-discordant -X ${maxins}"

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
