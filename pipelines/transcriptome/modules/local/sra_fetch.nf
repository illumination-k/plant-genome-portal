process SRA_FETCH {
    tag "${meta.id}:${sra_id}"
    label 'process_low'
    label 'process_long'

    container 'quay.io/biocontainers/sra-tools:3.1.1--h4304569_3'

    publishDir "${params.outdir}/sra", mode: params.publish_dir_mode,
        saveAs: { fn -> fn ==~ /.*\.fastq\.gz/ ? fn : null }

    input:
    tuple val(meta), val(sra_id)

    output:
    tuple val(meta), path("*.fastq.gz"), emit: reads
    path "versions.yml",                 emit: versions

    script:
    """
    set -euo pipefail

    prefetch --max-size 100g --progress ${sra_id}

    fasterq-dump \\
        --threads ${task.cpus} \\
        --split-files \\
        --skip-technical \\
        --temp \$PWD/tmp \\
        ${sra_id}

    rm -rf ${sra_id} tmp

    # Rename to sample.id-prefixed gz files so MultiQC groups them correctly
    for f in *.fastq; do
        case "\$f" in
            ${sra_id}_1.fastq) mv "\$f" "${meta.id}_1.fastq" ;;
            ${sra_id}_2.fastq) mv "\$f" "${meta.id}_2.fastq" ;;
            ${sra_id}.fastq)   mv "\$f" "${meta.id}.fastq"   ;;
        esac
    done

    pigz -p ${task.cpus} *.fastq

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        sra-tools: \$(prefetch --version 2>&1 | sed -n 's/^.*: //p' | head -n1)
    END_VERSIONS
    """
}
