use genome_domain::GoTermId;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SemSimError {
    #[error("unknown GO term: {0}")]
    UnknownTerm(GoTermId),
    #[error("term has no namespace: {0}")]
    MissingNamespace(GoTermId),
    #[error("cycle detected in GO DAG involving {0}")]
    CycleDetected(GoTermId),
}
