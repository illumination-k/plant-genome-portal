mod annotation;
mod error;
mod fasta;
mod gff;
mod kegg;
mod nomenclature;
mod obo;
mod protein;
mod repository;
mod snapshot;
mod util;

pub use crate::error::StorageError;
pub use crate::fasta::{FastaReference, refget_checksum};
pub use crate::kegg::KeggCatalogInput;
pub use crate::obo::{GoOntology, GoTerm, load_go_ontology};
pub use crate::repository::FileGenomeRepository;
pub use crate::snapshot::{
    GenomeSnapshot, GenomeSnapshotBuild, KeggCatalogPaths, KeggManifest, SnapshotManifest,
    build_genome_snapshot, read_snapshot, write_snapshot,
};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use genome_core::{Assembly, AssemblyAccession, FunctionalAnnotation, TaxId, Taxon};

    use super::*;

    #[test]
    fn snapshot_builder_parses_gene_transcript_and_exon() {
        let dir = tempfile::tempdir().unwrap();
        let fasta_path = dir.path().join("test.fa");
        let gff_path = dir.path().join("test.gff");

        let mut fasta = File::create(&fasta_path).unwrap();
        writeln!(fasta, ">chr1").unwrap();
        writeln!(fasta, "ACGTACGTACGT").unwrap();

        let mut gff = File::create(&gff_path).unwrap();
        writeln!(gff, "##gff-version 3").unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\tgene\t1\t8\t.\t+\t.\tID=Mp1g00010;Name=TEST"
        )
        .unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\tmRNA\t1\t8\t.\t+\t.\tID=Mp1g00010.1;Parent=Mp1g00010"
        )
        .unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\texon\t1\t4\t.\t+\t.\tParent=Mp1g00010.1"
        )
        .unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\tCDS\t2\t4\t.\t+\t0\tID=Mp1g00010.1.CDS;Parent=Mp1g00010.1"
        )
        .unwrap();

        let snapshot = build_genome_snapshot(&GenomeSnapshotBuild {
            fasta_path,
            gff_path,
            manifest: SnapshotManifest {
                source_base_url: "https://example.test".to_owned(),
                fasta_file: "test.fa".to_owned(),
                gff_file: "test.gff".to_owned(),
                functional_annotation_file: None,
                nomenclature_file: None,
                kegg_files: None,
                protein_fasta_file: None,
            },
            functional_annotation_path: None,
            nomenclature_path: None,
            protein_fasta_path: None,
            kegg_catalog_paths: Default::default(),
            taxon: Taxon {
                tax_id: TaxId::new(3197),
                scientific_name: "Marchantia polymorpha".to_owned(),
                common_name: None,
                rank: "species".to_owned(),
            },
            assembly: Assembly {
                accession: AssemblyAccession::new("GCA_test").unwrap(),
                tax_id: TaxId::new(3197),
                name: "test".to_owned(),
                source: genome_core::AssemblySource::Local,
                refget_checksum: None,
            },
        })
        .unwrap();

        assert_eq!(snapshot.dataset.sequences[0].length, 12);
        assert_eq!(
            snapshot.dataset.genes[0].assembly_accession.as_str(),
            "GCA_test"
        );
        assert_eq!(snapshot.dataset.genes[0].id.as_str(), "Mp1g00010");
        assert_eq!(snapshot.dataset.transcripts.len(), 1);
        assert_eq!(snapshot.dataset.exons.len(), 1);
        assert_eq!(snapshot.dataset.cdss.len(), 1);
        assert_eq!(snapshot.dataset.cdss[0].region.start.get(), 1);
        assert_eq!(snapshot.dataset.cdss[0].region.end.get(), 4);
        assert_eq!(snapshot.dataset.cdss[0].phase, Some(0));
    }

    #[test]
    fn snapshot_builder_imports_functional_annotation_and_nomenclature() {
        let dir = tempfile::tempdir().unwrap();
        let fasta_path = dir.path().join("test.fa");
        let gff_path = dir.path().join("test.gff");
        let functional_annotation_path = dir.path().join("func.tsv");
        let nomenclature_path = dir.path().join("nomenclature.tsv");

        let mut fasta = File::create(&fasta_path).unwrap();
        writeln!(fasta, ">chr1").unwrap();
        writeln!(fasta, "ACGTACGTACGT").unwrap();

        let mut gff = File::create(&gff_path).unwrap();
        writeln!(gff, "##gff-version 3").unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\tgene\t1\t8\t.\t+\t.\tID=Mp1g00010;Name=Mp1g00010"
        )
        .unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\tmRNA\t1\t8\t.\t+\t.\tID=Mp1g00010.1;Parent=Mp1g00010"
        )
        .unwrap();

        let mut functional_annotation = File::create(&functional_annotation_path).unwrap();
        writeln!(
            functional_annotation,
            "# This file contains annotation only from KEGG,KOG,Pfam,NCBIfam."
        )
        .unwrap();
        writeln!(
            functional_annotation,
            "Mp1g00010.1\tKEGG:K00001:example annotation; GO:0008150:biological_process; InterPro:IPR000001:example family; Pfam:PF00001:example domain; NCBIfam:NF000001:example family; KOG:KOG0001:example ortholog"
        )
        .unwrap();

        let mut nomenclature = File::create(&nomenclature_path).unwrap();
        writeln!(
            nomenclature,
            "gene_symbol\tfull_name\tsynonym\tproduct\tdescription\tGeneID/Location\treference\tPMID\tDOI\tstatus"
        )
        .unwrap();
        writeln!(
            nomenclature,
            "MpFOO\tFOO FULL\tMpBAR\ttranscription factor\tmanual note\tMapoly0001s0001.1; Mp1g00010.1\tExample reference\t12345\t10.0000/example\tPublished"
        )
        .unwrap();

        let snapshot = build_genome_snapshot(&GenomeSnapshotBuild {
            fasta_path,
            gff_path,
            manifest: SnapshotManifest {
                source_base_url: "https://example.test".to_owned(),
                fasta_file: "test.fa".to_owned(),
                gff_file: "test.gff".to_owned(),
                functional_annotation_file: Some("func.tsv".to_owned()),
                nomenclature_file: Some("nomenclature.tsv".to_owned()),
                kegg_files: None,
                protein_fasta_file: None,
            },
            functional_annotation_path: Some(functional_annotation_path),
            nomenclature_path: Some(nomenclature_path),
            protein_fasta_path: None,
            kegg_catalog_paths: Default::default(),
            taxon: Taxon {
                tax_id: TaxId::new(3197),
                scientific_name: "Marchantia polymorpha".to_owned(),
                common_name: None,
                rank: "species".to_owned(),
            },
            assembly: Assembly {
                accession: AssemblyAccession::new("GCA_test").unwrap(),
                tax_id: TaxId::new(3197),
                name: "test".to_owned(),
                source: genome_core::AssemblySource::Local,
                refget_checksum: None,
            },
        })
        .unwrap();

        let gene = &snapshot.dataset.genes[0];
        let transcript = &snapshot.dataset.transcripts[0];

        assert_eq!(gene.symbol.as_deref(), Some("MpFOO"));
        assert_eq!(
            gene.attributes
                .get("nomenclature_full_name")
                .map(String::as_str),
            Some("FOO FULL")
        );
        assert_eq!(
            gene.annotations
                .iter()
                .filter(|annotation| matches!(annotation, FunctionalAnnotation::Kegg(_)))
                .count(),
            1
        );
        assert_eq!(
            transcript
                .annotations
                .iter()
                .filter(|annotation| matches!(annotation, FunctionalAnnotation::GoTerm(_)))
                .count(),
            1
        );
        assert_eq!(
            transcript
                .annotations
                .iter()
                .filter(|annotation| matches!(annotation, FunctionalAnnotation::Pfam(_)))
                .count(),
            1
        );
        assert_eq!(
            transcript
                .annotations
                .iter()
                .filter(|annotation| matches!(annotation, FunctionalAnnotation::NcbiFam(_)))
                .count(),
            1
        );
        assert_eq!(
            transcript
                .annotations
                .iter()
                .filter(|annotation| matches!(annotation, FunctionalAnnotation::Kog(_)))
                .count(),
            1
        );
        assert_eq!(transcript.annotations.len(), 6);
    }

    #[test]
    fn snapshot_builder_attaches_protein_checksum_to_transcripts() {
        let dir = tempfile::tempdir().unwrap();
        let fasta_path = dir.path().join("test.fa");
        let gff_path = dir.path().join("test.gff");
        let protein_path = dir.path().join("proteins.fa");

        let mut fasta = File::create(&fasta_path).unwrap();
        writeln!(fasta, ">chr1").unwrap();
        writeln!(fasta, "ACGTACGTACGT").unwrap();

        let mut gff = File::create(&gff_path).unwrap();
        writeln!(gff, "##gff-version 3").unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\tgene\t1\t8\t.\t+\t.\tID=Mp1g00010;Name=TEST"
        )
        .unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\tmRNA\t1\t8\t.\t+\t.\tID=Mp1g00010.1;Parent=Mp1g00010"
        )
        .unwrap();

        let mut proteins = File::create(&protein_path).unwrap();
        writeln!(proteins, ">Mp1g00010.1").unwrap();
        writeln!(proteins, "MVTAGSMM").unwrap();

        let snapshot = build_genome_snapshot(&GenomeSnapshotBuild {
            fasta_path,
            gff_path,
            manifest: SnapshotManifest {
                source_base_url: "https://example.test".to_owned(),
                fasta_file: "test.fa".to_owned(),
                gff_file: "test.gff".to_owned(),
                functional_annotation_file: None,
                nomenclature_file: None,
                kegg_files: None,
                protein_fasta_file: Some("proteins.fa".to_owned()),
            },
            functional_annotation_path: None,
            nomenclature_path: None,
            protein_fasta_path: Some(protein_path),
            kegg_catalog_paths: Default::default(),
            taxon: Taxon {
                tax_id: TaxId::new(3197),
                scientific_name: "Marchantia polymorpha".to_owned(),
                common_name: None,
                rank: "species".to_owned(),
            },
            assembly: Assembly {
                accession: AssemblyAccession::new("GCA_test").unwrap(),
                tax_id: TaxId::new(3197),
                name: "test".to_owned(),
                source: genome_core::AssemblySource::Local,
                refget_checksum: None,
            },
        })
        .unwrap();

        let transcript = &snapshot.dataset.transcripts[0];
        assert_eq!(transcript.protein_length, Some(8));
        assert_eq!(
            transcript.protein_checksum.as_deref(),
            Some(refget_checksum(b"MVTAGSMM").as_str()),
        );
    }
}
