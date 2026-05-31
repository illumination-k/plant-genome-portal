process INTERPROSCAN {
    tag "${meta.id}"
    label 'process_high'

    container params.interproscan_container

    publishDir "${params.outdir}/interproscan/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(fasta), path(gene_metadata)

    output:
    tuple val(meta), path("${meta.id}.tsv"), path("${meta.id}.gff3"), emit: tsv
    path "versions.yml", emit: versions

    script:
    def appl = params.applications ? "-appl ${params.applications}" : ''
    """
    set -euo pipefail

    interproscan.sh \\
        -i ${fasta} \\
        -b ${meta.id} \\
        -f TSV,GFF3 \\
        -goterms \\
        -pa \\
        -cpu ${task.cpus} \\
        ${appl} \\
        ${params.interproscan_extra_args}

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        interproscan: \$(interproscan.sh -version 2>&1 | head -n 1)
    END_VERSIONS
    """
}
