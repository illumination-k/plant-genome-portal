process PREPARE_PROTEOME {
    tag "${meta.id}"
    label 'process_low'

    container params.python_container

    publishDir "${params.outdir}/prepared", mode: params.publish_dir_mode

    input:
    tuple val(meta), path(protein_fasta), path(gene_metadata)

    output:
    tuple val(meta), path("${meta.id}.fa"), path("${meta.id}.genes.tsv"), emit: prepared
    path "versions.yml", emit: versions

    script:
    """
    set -euo pipefail

    python - <<'PY'
    import gzip
    from pathlib import Path

    genome_id = "${meta.id}"
    tax_id = "${meta.tax_id}"
    scientific_name = "${meta.scientific_name}"
    assembly_accession = "${meta.assembly_accession}"
    protein_fasta = Path("${protein_fasta}")
    gene_metadata = Path("${gene_metadata}")

    gene_ids = {}
    symbols = {}
    if gene_metadata.name != "NO_FILE":
        with gene_metadata.open() as handle:
            header = handle.readline().rstrip("\\n").split("\\t")
            if "gene_id" not in header:
                raise SystemExit("gene_metadata_tsv must have a gene_id column")
            id_column = next((name for name in ["protein_id", "transcript_id", "sequence_id"] if name in header), "gene_id")
            id_idx = header.index(id_column)
            gene_idx = header.index("gene_id")
            symbol_idx = header.index("symbol") if "symbol" in header else None
            for line in handle:
                fields = line.rstrip("\\n").split("\\t")
                if len(fields) <= max(id_idx, gene_idx):
                    continue
                source_id = fields[id_idx]
                gene_id = fields[gene_idx]
                if not source_id or not gene_id:
                    continue
                gene_ids[source_id] = gene_id
                if symbol_idx is not None and len(fields) > symbol_idx:
                    symbols[source_id] = fields[symbol_idx]
                    symbols[gene_id] = fields[symbol_idx]

    opener = gzip.open if protein_fasta.name.endswith(".gz") else open
    genes = []
    with opener(protein_fasta, "rt") as src, open(f"{genome_id}.fa", "w") as fasta:
        for line in src:
            if line.startswith(">"):
                protein_id = line[1:].strip().split()[0]
                gene_id = gene_ids.get(protein_id, protein_id)
                genes.append((protein_id, gene_id))
                fasta.write(f">{protein_id}\\n")
            else:
                fasta.write(line)

    with open(f"{genome_id}.genes.tsv", "w") as out:
        out.write("genome_id\\ttax_id\\tscientific_name\\tassembly_accession\\tprotein_id\\tgene_id\\tsymbol\\n")
        for protein_id, gene_id in genes:
            symbol = symbols.get(protein_id, symbols.get(gene_id, ""))
            out.write(
                f"{genome_id}\\t{tax_id}\\t{scientific_name}\\t{assembly_accession}\\t"
                f"{protein_id}\\t{gene_id}\\t{symbol}\\n"
            )
    PY

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        python: \$(python --version | sed 's/^Python //')
    END_VERSIONS
    """
}
