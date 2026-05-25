use axum::{
    Json,
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{ApiError, AppState, ErrorResponse};

#[utoipa::path(
    get,
    path = "/sequence/service-info",
    operation_id = "refget_service_info",
    responses((status = 200, description = "refget service info", body = RefgetServiceInfo))
)]
pub(crate) async fn service_info() -> Json<RefgetServiceInfo> {
    Json(RefgetServiceInfo {
        id: "plant-genome-portal-refget".to_owned(),
        name: "Plant Genome Portal refget".to_owned(),
        circular_supported: false,
        subsequence_limit: None,
    })
}

#[utoipa::path(
    get,
    path = "/sequence/{checksum}",
    operation_id = "refget_sequence",
    params(
        ("checksum" = String, Path, description = "refget checksum"),
        RefgetQuery,
    ),
    responses(
        (status = 200, description = "Reference sequence", content_type = "text/vnd.ga4gh.refget.v2.0.0+plain", body = String),
        (status = 404, description = "Sequence not found", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
pub(crate) async fn sequence(
    State(state): State<AppState>,
    Path(checksum): Path<String>,
    Query(query): Query<RefgetQuery>,
) -> Result<impl IntoResponse, ApiError> {
    Ok((
        [
            (
                header::CONTENT_TYPE,
                "text/vnd.ga4gh.refget.v2.0.0+plain; charset=us-ascii",
            ),
            (header::ACCEPT_RANGES, "none"),
        ],
        state
            .service
            .refget_sequence(&checksum, query.start, query.end)?,
    ))
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub(crate) struct RefgetQuery {
    start: Option<u64>,
    end: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct RefgetServiceInfo {
    id: String,
    name: String,
    circular_supported: bool,
    subsequence_limit: Option<u64>,
}
