process QUALIMAP_RNASEQ {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/qualimap:2.3--hdfd78af_0'

    publishDir "${params.outdir}/qualimap/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(bam), path(bai)
    path gtf

    output:
    tuple val(meta), path("${meta.id}_rnaseq_qc"), emit: results
    path "versions.yml",                           emit: versions

    script:
    def strand = 'non-strand-specific'
    if (meta.strandedness == 'forward') strand = 'strand-specific-forward'
    if (meta.strandedness == 'reverse') strand = 'strand-specific-reverse'
    def pe_flag = meta.single_end ? '' : '-pe'
    def mem_g   = (task.memory.giga as int)
    """
    set -euo pipefail

    if [[ "${gtf}" == *.gz ]]; then
        zcat ${gtf} > annotation.gtf
    else
        cp ${gtf} annotation.gtf
    fi

    unset DISPLAY
    qualimap rnaseq \\
        -bam ${bam} \\
        -gtf annotation.gtf \\
        -outdir ${meta.id}_rnaseq_qc \\
        -p ${strand} \\
        ${pe_flag} \\
        --java-mem-size=${mem_g}G

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        qualimap: \$(qualimap --help 2>&1 | grep -oE 'QualiMap v\\.[0-9.]+' | head -n1 | sed 's/QualiMap v\\.//')
    END_VERSIONS
    """
}
