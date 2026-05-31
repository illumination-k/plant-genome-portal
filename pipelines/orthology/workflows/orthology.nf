include { PREPARE_PROTEOME      } from '../modules/local/prepare_proteome.nf'
include { ORTHOFINDER           } from '../modules/local/orthofinder.nf'
include { ORTHOGROUPS_TO_PORTAL } from '../modules/local/orthogroups_to_portal.nf'

workflow ORTHOLOGY {
    if (!params.genomes) error "Missing required parameter --genomes (CSV genome manifest)"

    ch_genomes = Channel.fromPath(params.genomes, checkIfExists: true)
        .splitCsv(header: true, strip: true)
        .map { row ->
            if (!row.genome_id)           error "genomes CSV row is missing genome_id"
            if (!row.tax_id)              error "genomes CSV row for ${row.genome_id} is missing tax_id"
            if (!row.scientific_name)     error "genomes CSV row for ${row.genome_id} is missing scientific_name"
            if (!row.assembly_accession)  error "genomes CSV row for ${row.genome_id} is missing assembly_accession"
            if (!row.protein_fasta)       error "genomes CSV row for ${row.genome_id} is missing protein_fasta"

            def meta = [
                id                : row.genome_id.trim(),
                tax_id            : row.tax_id.trim(),
                scientific_name   : row.scientific_name.trim(),
                assembly_accession: row.assembly_accession.trim(),
            ]
            def gene_metadata = row.gene_metadata_tsv && row.gene_metadata_tsv.trim()
                ? file(row.gene_metadata_tsv, checkIfExists: true)
                : file("${projectDir}/assets/NO_FILE")
            return [meta, file(row.protein_fasta, checkIfExists: true), gene_metadata]
        }

    PREPARE_PROTEOME(ch_genomes)

    ch_fastas = PREPARE_PROTEOME.out.prepared
        .map { meta, fasta, genes -> fasta }
        .collect()
    ch_gene_metadata = PREPARE_PROTEOME.out.prepared
        .map { meta, fasta, genes -> genes }
        .collect()

    ORTHOFINDER(ch_fastas)
    ORTHOGROUPS_TO_PORTAL(ORTHOFINDER.out.results, ch_gene_metadata)
}
