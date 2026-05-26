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
    path "${meta.id}.mito.txt",                                                    emit: mito_stats
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

    // Build the per-reference region list (everything except mito/plastid).
    // `samtools view <bam> <region> ...` restricts output to those references,
    // which is how we drop organelle reads.
    def mito_names = (params.mito_names ?: '').split(',').collect { it.trim() }.findAll { it }
    def mito_csv   = mito_names.join(',')
    """
    set -euo pipefail

    mito_pattern='${mito_csv}'

    # Refs to keep = all refs in the BAM minus the configured mito/plastid set.
    samtools view -H ${bam} \\
        | awk -v drop="\$mito_pattern" '
            BEGIN {
                n = split(drop, a, ",")
                for (i = 1; i <= n; i++) skip[a[i]] = 1
            }
            \$1 == "@SQ" {
                for (i = 2; i <= NF; i++) {
                    if (substr(\$i, 1, 3) == "SN:") {
                        name = substr(\$i, 4)
                        if (!(name in skip)) print name
                    }
                }
            }
        ' > keep.refs

        # Count mito reads (for QC) before filtering them out.
        for ref in \$(echo "\$mito_pattern" | tr ',' ' '); do
            count=\$(samtools view -c ${bam} "\$ref" 2>/dev/null || echo 0)
            echo -e "\$ref\\t\$count"
        done > ${meta.id}.mito.txt

    samtools view -@ ${task.cpus} -b \\
        -F ${flag_excl} ${flag_req} \\
        -q ${mapq} \\
        -o ${meta.id}.filt.bam \\
        ${bam} \$(cat keep.refs)

    samtools index    -@ ${task.cpus} ${meta.id}.filt.bam
    samtools flagstat -@ ${task.cpus} ${meta.id}.filt.bam > ${meta.id}.filt.flagstat

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        samtools: \$(samtools --version 2>&1 | head -n1 | sed 's/^samtools //')
    END_VERSIONS
    """
}
