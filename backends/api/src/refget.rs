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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::{chr1_checksum, sample_state};
    use axum::http::StatusCode;
    use genome_service::ServiceError;

    async fn body_string(response: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn service_info_advertises_the_portal_refget_service() {
        let Json(info) = service_info().await;

        assert_eq!(info.id, "plant-genome-portal-refget");
        assert_eq!(info.name, "Plant Genome Portal refget");
        assert!(!info.circular_supported);
        assert!(info.subsequence_limit.is_none());
    }

    #[tokio::test]
    async fn sequence_returns_full_bases_with_refget_content_type() {
        let response = sequence(
            State(sample_state()),
            Path(chr1_checksum()),
            Query(RefgetQuery {
                start: None,
                end: None,
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.contains("refget"));
        assert_eq!(body_string(response).await, "ACGTNNNNACGT");
    }

    #[tokio::test]
    async fn sequence_honours_the_start_and_end_range() {
        let response = sequence(
            State(sample_state()),
            Path(chr1_checksum()),
            Query(RefgetQuery {
                start: Some(0),
                end: Some(4),
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(body_string(response).await, "ACGT");
    }

    #[tokio::test]
    async fn sequence_maps_unknown_checksum_to_not_found() {
        let result = sequence(
            State(sample_state()),
            Path("not-a-real-checksum".to_owned()),
            Query(RefgetQuery {
                start: None,
                end: None,
            }),
        )
        .await;

        let Err(error) = result else {
            panic!("expected an error for an unknown checksum");
        };
        assert!(matches!(
            error,
            ApiError::Service(ServiceError::SequenceNotFound(_))
        ));
    }
}
