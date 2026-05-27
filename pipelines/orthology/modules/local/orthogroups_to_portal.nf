process ORTHOGROUPS_TO_PORTAL {
    label 'process_low'

    container params.python_container

    publishDir "${params.outdir}/portal", mode: params.publish_dir_mode

    input:
    path orthofinder_dir
    path gene_metadata, stageAs: "gene_metadata/*"

    output:
    path "orthogroups.tsv", emit: orthogroups
    path "versions.yml", emit: versions

    script:
    """
    set -euo pipefail

    python - <<'PY'
    import csv
    from pathlib import Path

    root = Path("${orthofinder_dir}")
    candidates = sorted(root.glob("**/Orthogroups/Orthogroups.tsv"))
    if not candidates:
        candidates = sorted(root.glob("**/Phylogenetic_Hierarchical_Orthogroups/N0.tsv"))
    if not candidates:
        raise SystemExit(f"could not find Orthogroups.tsv or N0.tsv under {root}")
    orthogroups = candidates[0]

    by_genome = {}
    by_protein = {}
    for path in sorted(Path("gene_metadata").glob("*.genes.tsv")):
        with path.open() as handle:
            reader = csv.DictReader(handle, delimiter="\\t")
            for row in reader:
                genome_id = row["genome_id"]
                by_genome[genome_id] = {
                    "tax_id": row["tax_id"],
                    "scientific_name": row["scientific_name"],
                    "assembly_accession": row["assembly_accession"],
                }
                protein_id = row.get("protein_id", row["gene_id"])
                by_protein[(genome_id, protein_id)] = (row["gene_id"], row.get("symbol", ""))

    rows = []
    with orthogroups.open() as src:
        reader = csv.DictReader(src, delimiter="\\t")
        group_column = reader.fieldnames[0]
        for row in reader:
            orthogroup_id = row[group_column]
            for genome_id in reader.fieldnames[1:]:
                meta = by_genome.get(genome_id)
                if meta is None:
                    continue
                value = row.get(genome_id, "")
                for protein_id in [part.strip() for part in value.split(",") if part.strip()]:
                    gene_id, symbol = by_protein.get((genome_id, protein_id), (protein_id, ""))
                    rows.append((
                        orthogroup_id,
                        meta["scientific_name"],
                        gene_id,
                        meta["tax_id"],
                        meta["assembly_accession"],
                        symbol,
                    ))

    with open("orthogroups.tsv", "w") as out:
        out.write("orthogroup_id\\tgene_id\\ttax_id\\tscientific_name\\tassembly_accession\\tsymbol\\n")
        for orthogroup_id, scientific_name, gene_id, tax_id, assembly_accession, symbol in sorted(rows):
            out.write(
                f"{orthogroup_id}\\t{gene_id}\\t{tax_id}\\t{scientific_name}\\t"
                f"{assembly_accession}\\t{symbol}\\n"
            )
    PY

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        python: \$(python --version | sed 's/^Python //')
    END_VERSIONS
    """
}
