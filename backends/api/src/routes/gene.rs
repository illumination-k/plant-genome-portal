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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::{TAX_ID, sample_state};
    use genome_service::ServiceError;

    fn search_query() -> GeneSearchQuery {
        GeneSearchQuery {
            tax_id: None,
            symbol: None,
            locus_tag: None,
            q: None,
            limit: None,
        }
    }

    #[tokio::test]
    async fn gene_returns_record_for_existing_id() {
        let Json(record) = gene(State(sample_state()), Path("Mp1g00010".to_owned()))
            .await
            .unwrap();

        assert_eq!(record.gene.id.as_str(), "Mp1g00010");
    }

    #[tokio::test]
    async fn gene_maps_missing_id_to_not_found() {
        let error = gene(State(sample_state()), Path("Mp9g99999".to_owned()))
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApiError::Service(ServiceError::GeneNotFound(_))
        ));
    }

    #[tokio::test]
    async fn gene_search_filters_by_symbol() {
        let Json(genes) = gene_search(
            State(sample_state()),
            Query(GeneSearchQuery {
                symbol: Some("FOO".to_owned()),
                ..search_query()
            }),
        )
        .await;

        assert_eq!(genes.len(), 1);
        assert_eq!(genes[0].id.as_str(), "Mp1g00010");
    }

    #[tokio::test]
    async fn gene_search_query_matches_substring_and_respects_limit() {
        let Json(matches) = gene_search(
            State(sample_state()),
            Query(GeneSearchQuery {
                q: Some("bar".to_owned()),
                ..search_query()
            }),
        )
        .await;
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id.as_str(), "Mp1g00020");

        let Json(limited) = gene_search(
            State(sample_state()),
            Query(GeneSearchQuery {
                limit: Some(1),
                ..search_query()
            }),
        )
        .await;
        assert_eq!(limited.len(), 1);
    }

    #[tokio::test]
    async fn gene_search_filters_out_other_taxa() {
        let Json(genes) = gene_search(
            State(sample_state()),
            Query(GeneSearchQuery {
                tax_id: Some(TAX_ID + 1),
                ..search_query()
            }),
        )
        .await;

        assert!(genes.is_empty());
    }

    #[tokio::test]
    async fn gene_orthogroups_is_empty_for_gene_without_membership() {
        let Json(groups) = gene_orthogroups(State(sample_state()), Path("Mp1g00010".to_owned()))
            .await
            .unwrap();

        assert!(groups.is_empty());
    }

    #[tokio::test]
    async fn orthogroup_maps_missing_id_to_not_found() {
        let error = orthogroup(State(sample_state()), Path("OG404".to_owned()))
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApiError::Service(ServiceError::OrthogroupNotFound(_))
        ));
    }

    #[tokio::test]
    async fn kegg_pathways_is_empty_without_catalog() {
        let Json(pathways) = kegg_pathways(State(sample_state())).await;

        assert!(pathways.is_empty());
    }

    #[tokio::test]
    async fn kegg_pathway_maps_missing_id_to_not_found() {
        let error = kegg_pathway(State(sample_state()), Path("map99999".to_owned()))
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApiError::Service(ServiceError::KeggPathwayNotFound(_))
        ));
    }
}
