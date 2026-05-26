process SALMON_INDEX {
    tag "${transcripts.simpleName}"
    label 'process_high'

    container 'quay.io/biocontainers/salmon:1.10.3--h6dccd9a_2'

    publishDir "${params.outdir}/salmon", mode: params.publish_dir_mode

    input:
    path transcripts
    path genome

    output:
    path "salmon_index", emit: index
    path "versions.yml", emit: versions

    script:
    """
    set -euo pipefail

    if [[ "${genome}" == *.gz ]]; then
        gunzip -c ${genome} > _genome.fa
    else
        cat ${genome} > _genome.fa
    fi
    if [[ "${transcripts}" == *.gz ]]; then
        gunzip -c ${transcripts} > _tx.fa
    else
        cat ${transcripts} > _tx.fa
    fi

    # decoy IDs = primary sequence headers in the genome FASTA
    grep '^>' _genome.fa | cut -d ' ' -f 1 | sed 's/^>//' > decoys.txt

    cat _tx.fa _genome.fa | gzip > gentrome.fa.gz

    salmon index \\
        --threads ${task.cpus} \\
        -t gentrome.fa.gz \\
        -d decoys.txt \\
        -i salmon_index \\
        -k 31

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        salmon: \$(salmon --version 2>&1 | sed 's/^salmon //')
    END_VERSIONS
    """
}
