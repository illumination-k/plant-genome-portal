process SAMTOOLS_FILTER {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/samtools:1.20--h50ea8bc_1'

    publishDir "${params.outdir}/filtered/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(bam), path(bai)

    output:
    tuple val(meta), path("${meta.id}.filt.bam"), path("${meta.id}.filt.bam.bai"), emit: bam
    path "${meta.id}.filt.flagstat",                                               emit: flagstat
    path "versions.yml",                                                           emit: versions

    script:
    // -F 0x004: drop unmapped
    // -F 0x100: drop secondary
    // -F 0x400: drop duplicates (unless keep_dups)
    // -F 0x800: drop supplementary
    // -f 0x002: keep only properly paired (PE only)
    def flag_excl = params.keep_dups ? (0x004 + 0x100 + 0x800) : (0x004 + 0x100 + 0x400 + 0x800)
    def flag_req  = meta.single_end ? '' : "-f ${0x002}"
    def mapq      = params.min_mapq ?: 0

    """
    set -euo pipefail

    samtools view -@ ${task.cpus} -b \\
        -F ${flag_excl} ${flag_req} \\
        -q ${mapq} \\
        -o ${meta.id}.filt.bam \\
        ${bam}

    samtools index -@ ${task.cpus} ${meta.id}.filt.bam
    samtools flagstat -@ ${task.cpus} ${meta.id}.filt.bam > ${meta.id}.filt.flagstat

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        samtools: \$(samtools --version 2>&1 | head -n1 | sed 's/^samtools //')
    END_VERSIONS
    """
}
