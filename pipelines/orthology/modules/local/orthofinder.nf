process ORTHOFINDER {
    label 'process_high'

    container params.orthofinder_container

    publishDir "${params.outdir}/orthofinder", mode: params.publish_dir_mode

    input:
    path fastas, stageAs: "input_fastas/*"

    output:
    path "orthofinder", emit: results
    path "versions.yml", emit: versions

    script:
    """
    set -euo pipefail

    orthofinder \\
        -f input_fastas \\
        -t ${task.cpus} \\
        -a ${task.cpus} \\
        -o orthofinder \\
        ${params.orthofinder_extra_args}

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        orthofinder: \$(orthofinder --version 2>&1 | sed 's/^.*version //')
    END_VERSIONS
    """
}
