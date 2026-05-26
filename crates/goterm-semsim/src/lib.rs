//! Semantic similarity between Gene Ontology terms.
//!
//! This crate is the pure-Rust core for GO semantic similarity. It owns
//! a small in-memory DAG ([`GoDag`]), two information-content sources
//! ([`IntrinsicIc`] and [`CorpusIc`]), and the four pairwise metrics most
//! commonly used in the literature: Resnik, Lin, Jiang-Conrath, and
//! Wang 2007. Set-level similarity ([`set_similarity`]) — Best Match
//! Average, Max, or Average — lets callers compare two gene's GO term
//! sets.
//!
//! No I/O, no async, no dependency on `storage` or `axum`: the DAG is
//! built via [`GoDag::builder`] from any source, including the OBO loader
//! in `storage`.
//!
//! ```
//! use genome_core::{GoNamespace, GoTermId};
//! use goterm_semsim::{GoDag, GoNode, IntrinsicIc, SimilarityMethod, similarity};
//!
//! let mut b = GoDag::builder();
//! b.insert(GoNode {
//!     id: GoTermId::new("GO:0008150").unwrap(),
//!     namespace: Some(GoNamespace::BiologicalProcess),
//!     is_a: vec![],
//!     part_of: vec![],
//! });
//! b.insert(GoNode {
//!     id: GoTermId::new("GO:0009987").unwrap(),
//!     namespace: Some(GoNamespace::BiologicalProcess),
//!     is_a: vec![GoTermId::new("GO:0008150").unwrap()],
//!     part_of: vec![],
//! });
//! let dag = b.build();
//! let ic = IntrinsicIc::from_dag(&dag);
//! let leaf = GoTermId::new("GO:0009987").unwrap();
//! assert!(similarity(&dag, &ic, &leaf, &leaf, SimilarityMethod::Lin).unwrap() > 0.0);
//! ```

mod dag;
mod error;
mod ic;
mod similarity;

pub use crate::dag::{GoDag, GoDagBuilder, GoNode};
pub use crate::error::SemSimError;
pub use crate::ic::{CorpusIc, InformationContent, IntrinsicIc};
pub use crate::similarity::{
    SetAggregator, SimilarityMethod, WangOptions, mica, set_similarity, similarity, wang,
};
