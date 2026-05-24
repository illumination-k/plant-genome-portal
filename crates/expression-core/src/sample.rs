use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

use genome_core::AssemblyAccession;

use crate::ids::{
    BioProjectAccession, BioSampleAccession, SraExperimentAccession, SraRunAccession,
    SraStudyAccession,
};

/// One quantification unit in an RNA-seq dataset, keyed by SRA Run accession.
///
/// We follow the convention used by ARCHS4 / refine.bio / Expression Atlas:
/// the sample primary key is the SRA Run (`SRR/ERR/DRR`), because one Run
/// corresponds to one FASTQ pair and therefore one expression quantification.
/// Parent accessions in the SRA hierarchy are carried alongside so callers can
/// group technical replicates by BioSample, or all samples in a study /
/// project.
///
/// Free-form fields (`tissue`, `developmental_stage`, ...) are intentionally
/// `Option<String>` — plant ontologies are diverse (PO, PECO, EFO, ENVO) and
/// locking in a controlled vocabulary at this layer would preclude downstream
/// reuse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Sample {
    /// SRA Run accession — the primary key for this sample.
    pub run: SraRunAccession,
    /// SRA Experiment (library construction event) the run belongs to.
    pub experiment: Option<SraExperimentAccession>,
    /// SRA Study the run belongs to.
    pub study: Option<SraStudyAccession>,
    /// BioSample — the biological material that was sequenced.
    pub biosample: Option<BioSampleAccession>,
    /// BioProject — the top-level umbrella the sample falls under.
    pub bioproject: Option<BioProjectAccession>,
    /// Assembly the quantification was performed against.
    pub assembly_accession: AssemblyAccession,
    pub title: Option<String>,
    pub tissue: Option<String>,
    pub developmental_stage: Option<String>,
    pub treatment: Option<String>,
    pub condition: Option<String>,
    pub replicate: Option<u32>,
    /// SRA `library_strategy` (e.g. `"RNA-Seq"`, `"miRNA-Seq"`).
    pub library_strategy: Option<String>,
    /// SRA `library_layout` (`"SINGLE"` or `"PAIRED"`).
    pub library_layout: Option<String>,
    /// SRA `platform` (e.g. `"ILLUMINA"`, `"OXFORD_NANOPORE"`).
    pub platform: Option<String>,
    /// SRA `instrument_model` (e.g. `"Illumina NovaSeq 6000"`).
    pub instrument_model: Option<String>,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

/// Top-level grouping aggregate corresponding to an INSDC BioProject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct BioProject {
    pub accession: BioProjectAccession,
    pub title: String,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}
