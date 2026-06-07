//! Shared fixtures for API handler tests.
//!
//! Builds an in-memory [`AppState`](crate::AppState) from a tiny genome dataset
//! plus a temp-file FASTA reference, so handler tests can exercise the real
//! wiring (service lookups, coordinate conversion, refget slicing) without a
//! snapshot on disk or a running server.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use genome_domain::{
    Assembly, AssemblyAccession, AssemblySource, Gene, GeneId, GenomeDataset, HalfOpenRegion,
    KeggCatalog, OrthogroupCatalog, Position0, Sequence, SequenceName, Strand, TaxId, Taxon,
};
use genome_service::GenomeService;
use genome_store::{FastaReference, FileGenomeRepository, refget_checksum};

use crate::{AppService, AppState};

pub(crate) const DEFAULT_ACCESSION: &str = "GCA_test";
pub(crate) const MISSING_ACCESSION: &str = "GCA_absent";
pub(crate) const TAX_ID: u32 = 3197;
pub(crate) const MISSING_TAX_ID: u32 = 999_999;
/// The bases backing `chr1`; the refget checksum of these is the only addressable
/// sequence in the fixture.
pub(crate) const CHR1_BASES: &[u8] = b"ACGTNNNNACGT";

pub(crate) fn chr1_checksum() -> String {
    refget_checksum(CHR1_BASES)
}

fn gene(id: &str, symbol: &str, start: u64, end: u64) -> Gene {
    Gene {
        id: GeneId::new(id).unwrap(),
        assembly_accession: AssemblyAccession::new(DEFAULT_ACCESSION).unwrap(),
        symbol: Some(symbol.to_owned()),
        locus_tag: None,
        sequence_name: SequenceName::new("chr1").unwrap(),
        region: HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(start),
            Position0::new(end),
        )
        .unwrap(),
        strand: Strand::Forward,
        feature_type: "gene".to_owned(),
        annotations: Vec::new(),
        attributes: BTreeMap::new(),
    }
}

fn sample_service() -> AppService {
    let accession = AssemblyAccession::new(DEFAULT_ACCESSION).unwrap();
    let dataset = GenomeDataset {
        taxon: Taxon {
            tax_id: TaxId::new(TAX_ID),
            scientific_name: "Marchantia polymorpha".to_owned(),
            common_name: None,
            rank: "species".to_owned(),
        },
        assembly: Assembly {
            accession: accession.clone(),
            tax_id: TaxId::new(TAX_ID),
            name: "test".to_owned(),
            source: AssemblySource::Local,
            refget_checksum: None,
        },
        sequences: vec![Sequence {
            name: SequenceName::new("chr1").unwrap(),
            assembly_accession: accession,
            length: CHR1_BASES.len() as u64,
            refget_checksum: chr1_checksum(),
        }],
        // Mp1g00010 covers the half-open [0, 10) (1-based 1-10); Mp1g00020 covers
        // [100, 200) (1-based 101-200). The gap lets region queries prove the
        // 1-based-closed to 0-based-half-open conversion.
        genes: vec![
            gene("Mp1g00010", "FOO", 0, 10),
            gene("Mp1g00020", "BAR", 100, 200),
        ],
        transcripts: Vec::new(),
        exons: Vec::new(),
        cdss: Vec::new(),
        kegg_catalog: KeggCatalog::default(),
        orthogroup_catalog: OrthogroupCatalog::default(),
    };

    GenomeService::new(FileGenomeRepository::new(dataset), Some(reference()))
}

fn reference() -> FastaReference {
    static NEXT_FASTA_ID: AtomicU64 = AtomicU64::new(1);
    let suffix = NEXT_FASTA_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "plant-genome-portal-api-test-{}-{suffix}.fa",
        std::process::id(),
    ));
    fs::write(&path, b">chr1\nACGTNNNNACGT\n").unwrap();
    let reference = FastaReference::from_path(&path).unwrap();
    fs::remove_file(&path).unwrap();
    reference
}

pub(crate) fn sample_state() -> AppState {
    AppState {
        service: sample_service(),
        expression_repository: None,
        epigenome_repository: None,
        epigenome_base_path: None,
        default_assembly_accession: DEFAULT_ACCESSION.to_owned(),
        blast_jobs: None,
        blastp_jobs: None,
    }
}
