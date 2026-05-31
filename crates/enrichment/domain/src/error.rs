#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EnrichmentError {
    #[error("population must be non-empty")]
    EmptyPopulation,
    #[error(
        "hypergeometric parameters out of range: population={population}, successes={successes}, draws={draws}"
    )]
    InvalidHypergeometric {
        population: u64,
        successes: u64,
        draws: u64,
    },
}
