use axum::{
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::{ApiError, AppState, ErrorResponse, sequence::SequenceOutputFormat};

const FASTA_LINE_LENGTH: usize = 60;

#[utoipa::path(
    get,
    path = "/v2/transcript/id/{transcript_id}/protein",
    operation_id = "transcript_protein",
    params(
        ("transcript_id" = String, Path, description = "Transcript identifier (e.g. Mp1g00010.1)"),
        ProteinQuery,
    ),
    responses(
        (status = 200, description = "Translated protein sequence", content_type = "text/plain", body = String),
        (status = 404, description = "Transcript or protein sequence not available", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
pub(crate) async fn sequence(
    State(state): State<AppState>,
    Path(transcript_id): Path<String>,
    Query(query): Query<ProteinQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let protein = state.service.transcript_protein(&transcript_id)?;
    let format = query.format.unwrap_or(SequenceOutputFormat::Plain);
    let body = match format {
        SequenceOutputFormat::Plain => protein.sequence,
        SequenceOutputFormat::Fasta => format_fasta(&protein.transcript_id, &protein.sequence),
    };
    let content_type = match format {
        SequenceOutputFormat::Plain => "text/plain; charset=us-ascii",
        SequenceOutputFormat::Fasta => "text/x-fasta; charset=us-ascii",
    };
    Ok(([(header::CONTENT_TYPE, content_type)], body))
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub(crate) struct ProteinQuery {
    /// Response format. Use `fasta` for sequence downloads (default `plain`).
    pub(crate) format: Option<SequenceOutputFormat>,
}

fn format_fasta(label: &str, sequence: &str) -> String {
    let mut body = format!(">{label}\n");
    for start in (0..sequence.len()).step_by(FASTA_LINE_LENGTH) {
        let end = (start + FASTA_LINE_LENGTH).min(sequence.len());
        body.push_str(&sequence[start..end]);
        body.push('\n');
    }
    body
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn format_fasta_wraps_sequence_with_transcript_id_header() {
        assert_eq!(
            format_fasta("Mp1g00010.1", "MVTAGSMM"),
            ">Mp1g00010.1\nMVTAGSMM\n"
        );
    }

    #[test]
    fn format_fasta_wraps_long_sequences_to_60_columns() {
        let sequence: String = "M".repeat(125);
        let body = format_fasta("X", &sequence);
        let lines = body.lines().collect::<Vec<_>>();
        assert_eq!(lines[0], ">X");
        assert_eq!(lines[1].len(), 60);
        assert_eq!(lines[2].len(), 60);
        assert_eq!(lines[3].len(), 5);
    }
}
