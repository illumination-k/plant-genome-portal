process FASTP {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/fastp:0.23.4--h125f33a_5'

    publishDir "${params.outdir}/fastp/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(reads)

    output:
    tuple val(meta), path("*.trim.fastq.gz"), emit: reads
    path "*.fastp.json",                      emit: json
    path "*.fastp.html",                      emit: html
    path "versions.yml",                      emit: versions

    script:
    def paired = !meta.single_end
    if (paired) {
        """
        fastp \\
            --in1 ${reads[0]} --in2 ${reads[1]} \\
            --out1 ${meta.id}_1.trim.fastq.gz \\
            --out2 ${meta.id}_2.trim.fastq.gz \\
            --json ${meta.id}.fastp.json \\
            --html ${meta.id}.fastp.html \\
            --detect_adapter_for_pe \\
            --thread ${task.cpus} \\
            2> ${meta.id}.fastp.log

        cat <<-END_VERSIONS > versions.yml
        "${task.process}":
            fastp: \$(fastp --version 2>&1 | sed 's/^fastp //')
        END_VERSIONS
        """
    } else {
        """
        fastp \\
            --in1 ${reads} \\
            --out1 ${meta.id}.trim.fastq.gz \\
            --json ${meta.id}.fastp.json \\
            --html ${meta.id}.fastp.html \\
            --thread ${task.cpus} \\
            2> ${meta.id}.fastp.log

        cat <<-END_VERSIONS > versions.yml
        "${task.process}":
            fastp: \$(fastp --version 2>&1 | sed 's/^fastp //')
        END_VERSIONS
        """
    }
}
