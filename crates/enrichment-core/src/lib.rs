//! Core enrichment-analysis statistics.
//!
//! This crate is the pure-statistical foundation that higher layers
//! (GO / Pfam / KEGG enrichment, web API, worker jobs) build on.
//! It has no I/O, no async, no ontology semantics — it only knows about
//! "items" (genes) drawn from a population, and "terms" that group items.
//!
//! Given a study set, a population (background) set, and a mapping from
//! `Term -> Vec<Item>`, [`run_enrichment`] computes, for each term:
//!
//! * the 2 x 2 contingency table restricted to the population,
//! * the one-sided (upper-tail) Fisher / hypergeometric `p`-value
//!   for over-representation,
//! * the fold enrichment,
//! * the Benjamini-Hochberg-adjusted `q`-value across the tested terms.
//!
//! The term and item types are generic so the same machinery can be
//! reused for GO terms, Pfam accessions, KEGG entries, or any other
//! categorical annotation.

mod analysis;
mod error;
mod fdr;
mod hypergeometric;
mod result;

pub use analysis::{EnrichmentInput, EnrichmentOptions, run_enrichment};
pub use error::EnrichmentError;
pub use fdr::benjamini_hochberg;
pub use hypergeometric::hypergeometric_upper_tail;
pub use result::EnrichmentResult;
