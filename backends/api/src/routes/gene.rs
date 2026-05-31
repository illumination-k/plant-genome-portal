use axum::{
    Json,
    extract::{Path, Query, State},
};
use genome_domain::{GeneSearch, TaxId};
use genome_service::{GeneKeggView, KeggPathwayDetail, KeggPathwaySummary};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::{ApiError, AppState};

#[utoipa::path(
    get,
    path = "/v2/gene/id/{gene_id}",
    params(("gene_id" = String, Path, description = "Gene identifier")),
    responses(
        (status = 200, description = "Gene detail", body = genome_domain::GeneRecord),
        (status = 404, description = "Gene not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn gene(
    State(state): State<AppState>,
    Path(gene_id): Path<String>,
) -> Result<Json<genome_domain::GeneRecord>, ApiError> {
    Ok(Json(state.service.gene(&gene_id)?))
}

#[utoipa::path(
    get,
    path = "/v2/gene/id/{gene_id}/kegg",
    params(("gene_id" = String, Path, description = "Gene identifier")),
    responses(
        (status = 200, description = "Per-gene KEGG view with KOs hydrated by their related pathways/modules/reactions", body = GeneKeggView),
        (status = 404, description = "Gene not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn gene_kegg(
    State(state): State<AppState>,
    Path(gene_id): Path<String>,
) -> Result<Json<GeneKeggView>, ApiError> {
    Ok(Json(state.service.gene_kegg_view(&gene_id)?))
}

#[utoipa::path(
    get,
    path = "/v2/gene/id/{gene_id}/orthogroups",
    params(("gene_id" = String, Path, description = "Gene identifier")),
    responses(
        (status = 200, description = "Orthogroups containing the gene", body = Vec<genome_domain::Orthogroup>),
        (status = 404, description = "Gene not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn gene_orthogroups(
    State(state): State<AppState>,
    Path(gene_id): Path<String>,
) -> Result<Json<Vec<genome_domain::Orthogroup>>, ApiError> {
    Ok(Json(state.service.gene_orthogroups(&gene_id)?))
}

#[utoipa::path(
    get,
    path = "/v2/orthogroup/{orthogroup_id}",
    params(("orthogroup_id" = String, Path, description = "Orthogroup identifier")),
    responses(
        (status = 200, description = "Orthogroup detail", body = genome_domain::Orthogroup),
        (status = 404, description = "Orthogroup not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn orthogroup(
    State(state): State<AppState>,
    Path(orthogroup_id): Path<String>,
) -> Result<Json<genome_domain::Orthogroup>, ApiError> {
    Ok(Json(state.service.orthogroup(&orthogroup_id)?))
}

#[utoipa::path(
    get,
    path = "/v2/kegg/pathways",
    responses(
        (status = 200, description = "KEGG pathway catalog with dataset-level KO and gene counts", body = Vec<KeggPathwaySummary>),
    )
)]
pub(crate) async fn kegg_pathways(State(state): State<AppState>) -> Json<Vec<KeggPathwaySummary>> {
    Json(state.service.kegg_pathways())
}

#[utoipa::path(
    get,
    path = "/v2/kegg/pathway/{pathway_id}",
    params(("pathway_id" = String, Path, description = "Canonical KEGG pathway id (e.g. map00010)")),
    responses(
        (status = 200, description = "KEGG pathway detail with KOs and the genes annotated with each KO", body = KeggPathwayDetail),
        (status = 404, description = "KEGG pathway not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn kegg_pathway(
    State(state): State<AppState>,
    Path(pathway_id): Path<String>,
) -> Result<Json<KeggPathwayDetail>, ApiError> {
    Ok(Json(state.service.kegg_pathway(&pathway_id)?))
}

#[utoipa::path(
    get,
    path = "/v2/gene/search",
    params(GeneSearchQuery),
    responses(
        (status = 200, description = "Matching genes", body = Vec<genome_domain::Gene>),
    )
)]
pub(crate) async fn gene_search(
    State(state): State<AppState>,
    Query(query): Query<GeneSearchQuery>,
) -> Json<Vec<genome_domain::Gene>> {
    Json(state.service.search_genes(query.into_search()))
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub(crate) struct GeneSearchQuery {
    tax_id: Option<u32>,
    symbol: Option<String>,
    locus_tag: Option<String>,
    q: Option<String>,
    limit: Option<usize>,
}

impl GeneSearchQuery {
    fn into_search(self) -> GeneSearch {
        GeneSearch {
            tax_id: self.tax_id.map(TaxId::new),
            symbol: self.symbol,
            locus_tag: self.locus_tag,
            query: self.q,
            limit: self.limit,
        }
    }
}
