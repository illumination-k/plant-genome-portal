process COMBINE_FUNCTIONAL_TSV {
    label 'process_low'

    container params.python_container

    publishDir "${params.outdir}/portal", mode: params.publish_dir_mode

    input:
    path functional_annotations, stageAs: "functional_annotations/*"

    output:
    path "functional_annotation.1_line.tsv", emit: combined
    path "versions.yml", emit: versions

    script:
    """
    set -euo pipefail

    cat functional_annotations/*.func_annotation.1_line.tsv > functional_annotation.1_line.tsv

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        python: \$(python --version | sed 's/^Python //')
    END_VERSIONS
    """
}
