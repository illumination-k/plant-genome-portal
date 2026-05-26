process BEDTOOLS_FRAGBEDGRAPH {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/bedtools:2.31.1--hf5e1c6e_2'

    publishDir "${params.outdir}/seacr/${meta.id}", mode: params.publish_dir_mode,
        pattern: '*.bedgraph'

    input:
    tuple val(meta), path(nsorted_bam)
    path chrom_sizes

    output:
    tuple val(meta), path("${meta.id}.fragments.bedgraph"), emit: bedgraph
    path "versions.yml",                                    emit: versions

    script:
    """
    set -euo pipefail

    # PE fragment intervals: keep pairs on same chr with insert <= 1000 (Henikoff SEACR recipe)
    bedtools bamtobed -bedpe -i ${nsorted_bam} \\
        | awk 'BEGIN{OFS="\\t"} \$1==\$4 && \$6-\$2 < 1000 {print \$1,\$2,\$6}' \\
        | sort -k1,1 -k2,2n -k3,3n \\
        > ${meta.id}.fragments.bed

    bedtools genomecov -bg -i ${meta.id}.fragments.bed -g ${chrom_sizes} \\
        > ${meta.id}.fragments.bedgraph

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        bedtools: \$(bedtools --version 2>&1 | sed 's/^bedtools v//')
    END_VERSIONS
    """
}
