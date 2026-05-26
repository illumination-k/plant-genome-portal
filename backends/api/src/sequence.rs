use axum::{
    extract::{Path, RawQuery, State},
    http::header,
    response::IntoResponse,
};
use genome_core::{HalfOpenRegion, Position0, SequenceName, Strand};
use serde::Serialize;
use service::ServiceError;
use utoipa::{IntoParams, ToSchema};

use crate::{ApiError, AppState, ErrorResponse};

#[utoipa::path(
    get,
    path = "/v2/genome/accession/{accession}/sequence/{sequence_name}",
    operation_id = "sequence_segments",
    params(
        ("accession" = String, Path, description = "Assembly accession"),
        ("sequence_name" = String, Path, description = "Sequence name, e.g. chr1"),
        SequenceSegmentsQuery,
    ),
    responses(
        (status = 200, description = "Concatenated sequence segments", content_type = "text/plain", body = String),
        (status = 404, description = "Assembly or sequence not found", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
pub(crate) async fn segments(
    State(state): State<AppState>,
    Path((accession, sequence_name)): Path<(String, String)>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, ApiError> {
    let query = parse_query(raw_query.as_deref(), &sequence_name)?;
    let sequence = state.service.sequence_segments_for_assembly(
        &accession,
        &sequence_name,
        query.segments.clone(),
        query.strand,
    )?;
    let body = match query.format {
        SequenceOutputFormat::Plain => sequence,
        SequenceOutputFormat::Fasta => format_fasta(
            &sequence_segments_label(&accession, &sequence_name, &query.segments, query.strand),
            &sequence,
        ),
    };
    let content_type = match query.format {
        SequenceOutputFormat::Plain => "text/plain; charset=us-ascii",
        SequenceOutputFormat::Fasta => "text/x-fasta; charset=us-ascii",
    };
    Ok(([(header::CONTENT_TYPE, content_type)], body))
}

#[allow(dead_code)]
#[derive(Debug, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub(crate) struct SequenceSegmentsQuery {
    /// Segment starts, 0-based inclusive. Repeat once per segment.
    start: Vec<u64>,
    /// Segment ends, 0-based exclusive. Repeat once per segment.
    end: Vec<u64>,
    /// Optional strand transform. `reverse` returns the reverse complement of the concatenated segments.
    strand: Option<Strand>,
    /// Response format. Use `fasta` for sequence downloads.
    format: Option<SequenceOutputFormat>,
}

#[derive(Debug)]
struct ParsedSequenceSegmentsQuery {
    format: SequenceOutputFormat,
    segments: Vec<HalfOpenRegion>,
    strand: Strand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SequenceOutputFormat {
    Plain,
    Fasta,
}

fn parse_query(
    raw_query: Option<&str>,
    sequence_name: &str,
) -> Result<ParsedSequenceSegmentsQuery, ApiError> {
    let mut starts = Vec::new();
    let mut ends = Vec::new();
    let mut format = SequenceOutputFormat::Plain;
    let mut strand = Strand::Forward;

    for (key, value) in raw_query
        .into_iter()
        .flat_map(|query| query.split('&'))
        .filter_map(|pair| {
            if pair.is_empty() {
                return None;
            }
            Some(pair.split_once('=').unwrap_or((pair, "")))
        })
    {
        match key {
            "start" | "start[]" => append_numbers(&mut starts, "start", value)?,
            "end" | "end[]" => append_numbers(&mut ends, "end", value)?,
            "format" => format = parse_format(value)?,
            "strand" => strand = parse_strand(value)?,
            _ => {}
        }
    }

    if starts.is_empty() || ends.is_empty() {
        return Err(invalid_request(
            "start and end must be provided at least once",
        ));
    }
    if starts.len() != ends.len() {
        return Err(invalid_request(
            "start and end must have the same number of values",
        ));
    }

    let sequence_name =
        SequenceName::new(sequence_name).map_err(|error| invalid_request(error.to_string()))?;
    let segments = starts
        .into_iter()
        .zip(ends)
        .map(|(start, end)| {
            HalfOpenRegion::new(
                sequence_name.clone(),
                Position0::new(start),
                Position0::new(end),
            )
            .map_err(|error| invalid_request(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ParsedSequenceSegmentsQuery {
        format,
        segments,
        strand,
    })
}

fn append_numbers(values: &mut Vec<u64>, field: &str, raw_value: &str) -> Result<(), ApiError> {
    for raw_number in raw_value.split(',') {
        let value = raw_number
            .parse::<u64>()
            .map_err(|_| invalid_request(format!("{field} must contain unsigned integers")))?;
        values.push(value);
    }
    Ok(())
}

fn parse_strand(raw_value: &str) -> Result<Strand, ApiError> {
    match raw_value {
        "forward" | "+" => Ok(Strand::Forward),
        "reverse" | "-" => Ok(Strand::Reverse),
        "unknown" | "." | "?" => Ok(Strand::Unknown),
        _ => Err(invalid_request(
            "strand must be one of forward, reverse, or unknown",
        )),
    }
}

fn parse_format(raw_value: &str) -> Result<SequenceOutputFormat, ApiError> {
    match raw_value {
        "plain" | "text" => Ok(SequenceOutputFormat::Plain),
        "fasta" | "fa" => Ok(SequenceOutputFormat::Fasta),
        _ => Err(invalid_request("format must be one of plain or fasta")),
    }
}

fn sequence_segments_label(
    accession: &str,
    sequence_name: &str,
    segments: &[HalfOpenRegion],
    strand: Strand,
) -> String {
    let ranges = segments
        .iter()
        .map(|segment| format!("{}-{}", segment.start.get(), segment.end.get()))
        .collect::<Vec<_>>()
        .join(",");
    format!("{accession}|{sequence_name}:{ranges}|strand={strand:?}")
}

fn format_fasta(label: &str, sequence: &str) -> String {
    const FASTA_LINE_LENGTH: usize = 60;
    let mut body = format!(">{label}\n");
    for start in (0..sequence.len()).step_by(FASTA_LINE_LENGTH) {
        let end = (start + FASTA_LINE_LENGTH).min(sequence.len());
        body.push_str(&sequence[start..end]);
        body.push('\n');
    }
    body
}

fn invalid_request(message: impl Into<String>) -> ApiError {
    ServiceError::InvalidRequest(message.into()).into()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_query_accepts_repeated_start_end_pairs() {
        let query = parse_query(
            Some("start=0&end=4&start=8&end=12&strand=reverse&format=fasta"),
            "chr1",
        )
        .unwrap();

        assert_eq!(query.format, SequenceOutputFormat::Fasta);
        assert_eq!(query.segments.len(), 2);
        assert_eq!(query.segments[0].start.get(), 0);
        assert_eq!(query.segments[0].end.get(), 4);
        assert_eq!(query.segments[1].start.get(), 8);
        assert_eq!(query.segments[1].end.get(), 12);
        assert_eq!(query.strand, Strand::Reverse);
    }

    #[test]
    fn parse_query_rejects_mismatched_start_end_counts() {
        let error = parse_query(Some("start=0&end=4&start=8"), "chr1").unwrap_err();

        assert!(matches!(
            error,
            ApiError::Service(ServiceError::InvalidRequest(_))
        ));
    }

    #[test]
    fn format_fasta_wraps_sequence_with_header() {
        assert_eq!(format_fasta("chr1:0-4", "ACGT"), ">chr1:0-4\nACGT\n");
    }
}
