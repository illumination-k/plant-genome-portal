//! Core genome domain types.
//!
//! This crate intentionally has no I/O or async dependencies.

mod annotation;
mod assembly;
mod coord;
mod error;
mod feature;
mod homology;
mod ids;
mod repository;

pub use annotation::{
    AnnotationEvidence, AnnotationSource, FunctionalAnnotation, GoNamespace, GoTermAnnotation,
    InterProAnnotation, KeggAnnotation, KeggEntryKind, KogAnnotation, NcbiFamAnnotation,
    PfamAnnotation,
};
pub use assembly::{Assembly, AssemblySource, Sequence, Taxon};
pub use coord::{ClosedRegion, HalfOpenRegion, Position0, Position1, Strand};
pub use error::DomainError;
pub use feature::{Cds, Exon, Gene, GeneRecord, GenomeDataset, Transcript};
pub use homology::{HomologyHit, HomologySearchMethod, HomologySearchResult};
pub use ids::{
    AssemblyAccession, GeneId, GoTermId, InterProId, KeggEntryId, KogEntryId, NcbiFamAccession,
    PfamAccession, SequenceName, TaxId, TranscriptId,
};
pub use repository::{GeneSearch, GenomeRepository};
