process BOWTIE2_BUILD {
    tag "${genome.simpleName}"
    label 'process_high'

    container 'quay.io/biocontainers/bowtie2:2.5.4--he20e202_2'

    publishDir "${params.outdir}/bowtie2", mode: params.publish_dir_mode

    input:
    path genome

    output:
    path "bowtie2_index", emit: index
    path "versions.yml",  emit: versions

    script:
    """
    set -euo pipefail

    mkdir -p bowtie2_index

    if [[ "${genome}" == *.gz ]]; then
        zcat ${genome} > genome.fa
    else
        cp ${genome} genome.fa
    fi

    bowtie2-build --threads ${task.cpus} genome.fa bowtie2_index/genome

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        bowtie2: \$(bowtie2 --version 2>&1 | head -n1 | sed 's/.* version //')
    END_VERSIONS
    """
}
