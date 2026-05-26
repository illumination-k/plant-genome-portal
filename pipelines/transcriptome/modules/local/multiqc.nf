process MULTIQC {
    label 'process_low'

    container 'quay.io/biocontainers/multiqc:1.24.1--pyhdfd78af_0'

    publishDir "${params.outdir}/multiqc", mode: params.publish_dir_mode

    input:
    path multiqc_files, stageAs: "?/*"
    path multiqc_config

    output:
    path "multiqc_report.html", emit: report
    path "multiqc_data",        emit: data
    path "versions.yml",        emit: versions

    script:
    def config_arg = multiqc_config.name != 'NO_FILE' ? "-c ${multiqc_config}" : ''
    """
    multiqc -f ${config_arg} .

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        multiqc: \$(multiqc --version 2>&1 | sed 's/^multiqc, version //')
    END_VERSIONS
    """
}
