use axum::{
    Json,
    extract::{Path, State},
};
use genome_core::TaxId;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{ApiError, AppState};

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct TaxonResponse {
    taxon: genome_core::Taxon,
    assemblies: Vec<genome_core::Assembly>,
}

#[utoipa::path(
    get,
    path = "/v2/genome/accession/{accession}",
    params(("accession" = String, Path, description = "Assembly accession")),
    responses(
        (status = 200, description = "Assembly metadata", body = genome_core::Assembly),
        (status = 404, description = "Assembly not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn assembly(
    State(state): State<AppState>,
    Path(accession): Path<String>,
) -> Result<Json<genome_core::Assembly>, ApiError> {
    Ok(Json(state.service.assembly(&accession)?))
}

#[utoipa::path(
    get,
    path = "/v2/genome/accession/{accession}/sequences",
    params(("accession" = String, Path, description = "Assembly accession")),
    responses(
        (status = 200, description = "Assembly sequences", body = Vec<genome_core::Sequence>),
        (status = 404, description = "Assembly not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn assembly_sequences(
    State(state): State<AppState>,
    Path(accession): Path<String>,
) -> Result<Json<Vec<genome_core::Sequence>>, ApiError> {
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
        (status = 200, description = "Overlapping genes", body = Vec<genome_core::Gene>),
        (status = 404, description = "Assembly not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn region_features(
    State(state): State<AppState>,
    Path((accession, region)): Path<(String, String)>,
) -> Result<Json<Vec<genome_core::Gene>>, ApiError> {
    Ok(Json(state.service.features_in_region(&accession, &region)?))
}
