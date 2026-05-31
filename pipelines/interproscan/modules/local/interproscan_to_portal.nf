process INTERPROSCAN_TO_PORTAL {
    tag "${meta.id}"
    label 'process_low'

    container params.python_container

    publishDir "${params.outdir}/portal/${meta.id}", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(interpro_tsv), path(interpro_gff3)

    output:
    path "${meta.id}.func_annotation.1_line.tsv", emit: functional_annotation
    path "versions.yml", emit: versions

    script:
    """
    set -euo pipefail

    python - <<'PY'
    import csv
    from collections import OrderedDict
    from pathlib import Path

    interpro_tsv = Path("${interpro_tsv}")
    annotations = OrderedDict()

    def add(seq_id, value):
        if not value:
            return
        annotations.setdefault(seq_id, OrderedDict())
        annotations[seq_id][value] = None

    with interpro_tsv.open() as handle:
        reader = csv.reader(handle, delimiter="\\t")
        for fields in reader:
            if len(fields) < 13:
                continue
            seq_id = fields[0]
            analysis = fields[3]
            signature = fields[4]
            signature_desc = fields[5] if len(fields) > 5 else ""
            interpro_id = fields[11] if len(fields) > 11 else ""
            interpro_desc = fields[12] if len(fields) > 12 else ""
            go_terms = fields[13] if len(fields) > 13 else ""

            if interpro_id.startswith("IPR"):
                add(seq_id, f"InterPro:{interpro_id}:{interpro_desc}")
            if analysis == "Pfam" or signature.startswith("PF"):
                add(seq_id, f"Pfam:{signature}:{signature_desc}")
            if analysis == "NCBIfam" or signature.startswith("NF"):
                add(seq_id, f"NCBIfam:{signature}:{signature_desc}")
            if analysis == "KOG" or signature.startswith("KOG"):
                add(seq_id, f"KOG:{signature}:{signature_desc}")
            for go in [part.strip() for part in go_terms.split("|") if part.strip()]:
                add(seq_id, go)

    with open("${meta.id}.func_annotation.1_line.tsv", "w") as out:
        for seq_id, values in sorted(annotations.items()):
            out.write(f"{seq_id}\\t{'; '.join(values.keys())}\\n")
    PY

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        python: \$(python --version | sed 's/^Python //')
    END_VERSIONS
    """
}
