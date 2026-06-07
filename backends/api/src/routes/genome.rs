use axum::{
    Json,
    extract::{Path, State},
};
use genome_domain::TaxId;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{ApiError, AppState};

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct TaxonResponse {
    taxon: genome_domain::Taxon,
    assemblies: Vec<genome_domain::Assembly>,
}

#[utoipa::path(
    get,
    path = "/v2/genome/accession/{accession}",
    params(("accession" = String, Path, description = "Assembly accession")),
    responses(
        (status = 200, description = "Assembly metadata", body = genome_domain::Assembly),
        (status = 404, description = "Assembly not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn assembly(
    State(state): State<AppState>,
    Path(accession): Path<String>,
) -> Result<Json<genome_domain::Assembly>, ApiError> {
    Ok(Json(state.service.assembly(&accession)?))
}

#[utoipa::path(
    get,
    path = "/v2/genome/accession/{accession}/sequences",
    params(("accession" = String, Path, description = "Assembly accession")),
    responses(
        (status = 200, description = "Assembly sequences", body = Vec<genome_domain::Sequence>),
        (status = 404, description = "Assembly not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn assembly_sequences(
    State(state): State<AppState>,
    Path(accession): Path<String>,
) -> Result<Json<Vec<genome_domain::Sequence>>, ApiError> {
    Ok(Json(state.service.sequences_for_assembly(&accession)?))
}

#[utoipa::path(
    get,
    path = "/v2/genome/taxon/{tax_id}",
    params(("tax_id" = u32, Path, description = "NCBI Taxonomy ID")),
    responses(
        (status = 200, description = "Taxon and assemblies", body = TaxonResponse),
        (status = 404, description = "Taxon not found", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn taxon(
    State(state): State<AppState>,
    Path(tax_id): Path<u32>,
) -> Result<Json<TaxonResponse>, ApiError> {
    let tax_id = TaxId::new(tax_id);
    Ok(Json(TaxonResponse {
        taxon: state.service.taxon(tax_id)?,
        assemblies: state.service.assemblies_for_taxon(tax_id),
    }))
}

#[utoipa::path(
    get,
    path = "/v2/genome/accession/{accession}/region/{region}/features",
    params(
        ("accession" = String, Path, description = "Assembly accession"),
        ("region" = String, Path, description = "1-based closed region, e.g. chr1:1-100000"),
    ),
    responses(
        (status = 200, description = "Overlapping genes", body = Vec<genome_domain::Gene>),
        (status = 404, description = "Assembly not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn region_features(
    State(state): State<AppState>,
    Path((accession, region)): Path<(String, String)>,
) -> Result<Json<Vec<genome_domain::Gene>>, ApiError> {
    Ok(Json(state.service.features_in_region(&accession, &region)?))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::{
        DEFAULT_ACCESSION, MISSING_ACCESSION, MISSING_TAX_ID, TAX_ID, sample_state,
    };
    use genome_service::ServiceError;

    fn gene_ids(genes: &[genome_domain::Gene]) -> Vec<&str> {
        genes.iter().map(|gene| gene.id.as_str()).collect()
    }

    #[tokio::test]
    async fn assembly_returns_metadata_for_known_accession() {
        let Json(assembly) = assembly(State(sample_state()), Path(DEFAULT_ACCESSION.to_owned()))
            .await
            .unwrap();

        assert_eq!(assembly.accession.as_str(), DEFAULT_ACCESSION);
    }

    #[tokio::test]
    async fn assembly_maps_unknown_accession_to_not_found() {
        let error = assembly(State(sample_state()), Path(MISSING_ACCESSION.to_owned()))
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApiError::Service(ServiceError::AssemblyNotFound(_))
        ));
    }

    #[tokio::test]
    async fn assembly_sequences_lists_the_sequences() {
        let Json(sequences) =
            assembly_sequences(State(sample_state()), Path(DEFAULT_ACCESSION.to_owned()))
                .await
                .unwrap();

        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].name.as_str(), "chr1");
    }

    #[tokio::test]
    async fn taxon_returns_taxon_and_its_assemblies() {
        let Json(response) = taxon(State(sample_state()), Path(TAX_ID)).await.unwrap();

        assert_eq!(response.taxon.tax_id, TaxId::new(TAX_ID));
        assert_eq!(response.assemblies.len(), 1);
        assert_eq!(response.assemblies[0].accession.as_str(), DEFAULT_ACCESSION);
    }

    #[tokio::test]
    async fn taxon_maps_unknown_tax_id_to_not_found() {
        let error = taxon(State(sample_state()), Path(MISSING_TAX_ID))
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApiError::Service(ServiceError::TaxonNotFound(_))
        ));
    }

    #[tokio::test]
    async fn region_features_converts_1_based_closed_bounds() {
        // 1-based closed chr1:1-10 maps to half-open [0, 10), which is exactly
        // Mp1g00010's span and must not touch Mp1g00020 at [100, 200).
        let Json(genes) = region_features(
            State(sample_state()),
            Path((DEFAULT_ACCESSION.to_owned(), "chr1:1-10".to_owned())),
        )
        .await
        .unwrap();

        assert_eq!(gene_ids(&genes), ["Mp1g00010"]);
    }

    #[tokio::test]
    async fn region_features_excludes_genes_just_outside_the_window() {
        // chr1:1-100 (half-open [0, 100)) stops one base short of Mp1g00020's
        // 0-based start at 100, so only the first gene overlaps.
        let Json(genes) = region_features(
            State(sample_state()),
            Path((DEFAULT_ACCESSION.to_owned(), "chr1:1-100".to_owned())),
        )
        .await
        .unwrap();

        assert_eq!(gene_ids(&genes), ["Mp1g00010"]);
    }

    #[tokio::test]
    async fn region_features_spanning_window_returns_both_genes() {
        let Json(genes) = region_features(
            State(sample_state()),
            Path((DEFAULT_ACCESSION.to_owned(), "chr1:1-200".to_owned())),
        )
        .await
        .unwrap();

        assert_eq!(gene_ids(&genes), ["Mp1g00010", "Mp1g00020"]);
    }

    #[tokio::test]
    async fn region_features_rejects_a_malformed_region() {
        let error = region_features(
            State(sample_state()),
            Path((DEFAULT_ACCESSION.to_owned(), "not-a-region".to_owned())),
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            ApiError::Service(ServiceError::InvalidRequest(_))
        ));
    }

    #[tokio::test]
    async fn region_features_maps_unknown_assembly_to_not_found() {
        let error = region_features(
            State(sample_state()),
            Path((MISSING_ACCESSION.to_owned(), "chr1:1-10".to_owned())),
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            ApiError::Service(ServiceError::AssemblyNotFound(_))
        ));
    }
}
