process HISAT2_BUILD {
    tag "${genome.simpleName}"
    label 'process_high'

    container 'quay.io/biocontainers/hisat2:2.2.1--h87f3376_5'

    publishDir "${params.outdir}/hisat2", mode: params.publish_dir_mode

    input:
    path genome
    path gtf

    output:
    path "hisat2_index", emit: index
    path "versions.yml", emit: versions

    script:
    """
    set -euo pipefail

    mkdir -p hisat2_index

    if [[ "${genome}" == *.gz ]]; then
        zcat ${genome} > genome.fa
    else
        cp ${genome} genome.fa
    fi

    if [[ "${gtf}" == *.gz ]]; then
        zcat ${gtf} > annotation.gtf
    else
        cp ${gtf} annotation.gtf
    fi

    hisat2_extract_splice_sites.py annotation.gtf > splice_sites.txt
    hisat2_extract_exons.py annotation.gtf > exons.txt

    hisat2-build \\
        -p ${task.cpus} \\
        --ss splice_sites.txt \\
        --exon exons.txt \\
        genome.fa hisat2_index/genome

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        hisat2: \$(hisat2 --version 2>&1 | head -n1 | sed 's/.*hisat2-align-s version //')
    END_VERSIONS
    """
}
