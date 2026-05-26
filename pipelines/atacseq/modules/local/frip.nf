process FRIP {
    tag "${meta.id}"
    label 'process_low'

    container 'quay.io/biocontainers/samtools:1.20--h50ea8bc_1'

    publishDir "${params.outdir}/frip", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(bam), path(bai), path(peaks)

    output:
    tuple val(meta), path("${meta.id}.frip.tsv"), emit: tsv
    path "versions.yml",                          emit: versions

    script:
    // FRiP = reads overlapping any peak / total mapped reads after filtering.
    // ENCODE QC threshold: FRiP >= 0.2 for high-quality ATAC libraries.
    """
    set -euo pipefail

    total=\$(samtools view -c -F 0x004 ${bam})
    in_peaks=\$(samtools view -c -F 0x004 -L ${peaks} ${bam})

    if [ "\$total" -gt 0 ]; then
        frip=\$(awk -v t="\$total" -v p="\$in_peaks" 'BEGIN { printf "%.6f", p/t }')
    else
        frip=0
    fi

    {
        echo -e "sample\\ttotal_reads\\treads_in_peaks\\tfrip"
        echo -e "${meta.id}\\t\$total\\t\$in_peaks\\t\$frip"
    } > ${meta.id}.frip.tsv

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        samtools: \$(samtools --version 2>&1 | head -n1 | sed 's/^samtools //')
    END_VERSIONS
    """
}
