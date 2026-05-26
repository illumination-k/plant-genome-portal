process HOMER_FINDMOTIFS {
    tag "${meta.id}"
    label 'process_medium'

    container 'quay.io/biocontainers/homer:5.1--pl5321hc52dbad_1'

    publishDir "${params.outdir}/homer/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(peaks)
    path genome

    output:
    tuple val(meta), path("${meta.id}_homer"), emit: results
    path "versions.yml",                       emit: versions

    script:
    // findMotifsGenome.pl runs both known motif enrichment (-mset auto: plants
    // when available) and de novo discovery in one pass.
    def size       = params.homer_size       ?: 200
    def mset       = params.homer_mset       ? "-mset ${params.homer_mset}" : ''
    def extra_args = params.homer_extra_args ?: '-mask'
    """
    set -euo pipefail

    if [[ "${genome}" == *.gz ]]; then
        zcat ${genome} > genome.fa
    else
        cp ${genome} genome.fa
    fi

    findMotifsGenome.pl \\
        ${peaks} \\
        genome.fa \\
        ${meta.id}_homer \\
        -size ${size} \\
        -p ${task.cpus} \\
        ${mset} \\
        ${extra_args}

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        homer: \$(findMotifsGenome.pl 2>&1 | grep -oE 'HOMER v[0-9.]+' | head -n1 | sed 's/HOMER v//')
    END_VERSIONS
    """
}
