use axum::{
    Json,
    extract::{Path, Query, State},
};
use epigenome_domain::{
    Assay, EpigenomeRepository, Experiment, ExperimentId, ExperimentQuery, Peak, PeakHit,
    PeakRegionQuery, Target,
};
use genome_domain::{AssemblyAccession, ClosedRegion};
use genome_service::ServiceError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};

use crate::{ApiError, AppState};

/// Maximum number of `experimentIds` accepted in a single query — keeps the
/// query string a sane length and prevents accidentally huge requests.
const MAX_EXPERIMENT_IDS: usize = 50;
const DEFAULT_PEAK_LIMIT: usize = 1_000;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExperimentListQuery {
    assembly_accession: Option<String>,
    assay: Option<Assay>,
    target: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpigenomePeaksQuery {
    assembly_accession: String,
    /// 1-based closed region, e.g. `chr1:1-100000`.
    region: String,
    experiment_ids: Option<String>,
    assay: Option<Assay>,
    target: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeneEpigenomeQuery {
    #[serde(default = "default_upstream_bp")]
    upstream_bp: u64,
    #[serde(default)]
    downstream_bp: u64,
    assay: Option<Assay>,
    target: Option<String>,
    limit: Option<usize>,
}

fn default_upstream_bp() -> u64 {
    2_000
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpigenomeExperimentSummary {
    pub experiment_id: String,
    pub assay: Assay,
    pub target: Option<String>,
    pub tissue: Option<String>,
    pub dev_stage: Option<String>,
    pub treatment: Option<String>,
    pub replicate: Option<u16>,
    pub peak_kind: epigenome_domain::PeakKind,
    pub frip: Option<f64>,
    pub signal_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpigenomeExperimentDetail {
    pub experiment: Experiment,
    pub peak_count: usize,
    pub signal_url: Option<String>,
}

/// A peak emitted on the public API: coordinates are re-projected to 1-based
/// closed form so the wire shape matches the portal's other region payloads.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PublicPeak {
    pub sequence_name: String,
    /// 1-based, inclusive.
    pub start: u64,
    /// 1-based, inclusive.
    pub end: u64,
    pub name: String,
    pub score: u16,
    pub strand: genome_domain::Strand,
    pub signal_value: f64,
    pub p_value: f64,
    pub q_value: f64,
    pub summit_offset: Option<u32>,
}

impl From<Peak> for PublicPeak {
    fn from(peak: Peak) -> Self {
        Self {
            sequence_name: peak.region.sequence_name.into_string(),
            // half-open [start, end) → 1-based closed [start+1, end]
            start: peak.region.start.get() + 1,
            end: peak.region.end.get(),
            name: peak.name,
            score: peak.score,
            strand: peak.strand,
            signal_value: peak.signal_value,
            p_value: peak.p_value,
            q_value: peak.q_value,
            summit_offset: peak.summit_offset,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpigenomePeakHit {
    pub experiment_id: String,
    pub peak: PublicPeak,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpigenomeExperimentWithPeaks {
    pub experiment: EpigenomeExperimentSummary,
    pub peaks: Vec<PublicPeak>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpigenomeGeneView {
    pub gene_id: String,
    pub assembly_accession: String,
    pub region: PublicRegion,
    pub experiments: Vec<EpigenomeExperimentWithPeaks>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PublicRegion {
    pub sequence_name: String,
    pub start: u64,
    pub end: u64,
}

#[utoipa::path(
    get,
    path = "/v2/epigenome/experiments",
    operation_id = "epigenome_experiments",
    params(ExperimentListQuery),
    responses(
        (status = 200, description = "Epigenome experiments matching the filters", body = Vec<EpigenomeExperimentSummary>),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn experiments(
    State(state): State<AppState>,
    Query(query): Query<ExperimentListQuery>,
) -> Result<Json<Vec<EpigenomeExperimentSummary>>, ApiError> {
    let Some(repository) = state.epigenome_repository.as_ref() else {
        return Ok(Json(Vec::new()));
    };

    let assembly_accession = query
        .assembly_accession
        .as_deref()
        .map(parse_assembly)
        .transpose()?;
    let target = query.target.as_deref().map(parse_target).transpose()?;

    let experiments = repository.experiments(&ExperimentQuery {
        assembly_accession,
        assay: query.assay,
        target,
        limit: query.limit,
    });

    let summaries = experiments
        .into_iter()
        .map(|experiment| {
            let signal_url = signal_url_for(&experiment, state.epigenome_base_path.as_deref());
            experiment_summary(experiment, signal_url)
        })
        .collect();
    Ok(Json(summaries))
}

#[utoipa::path(
    get,
    path = "/v2/epigenome/experiment/{experiment_id}",
    operation_id = "epigenome_experiment",
    params(("experiment_id" = String, Path, description = "Portal-local experiment id")),
    responses(
        (status = 200, description = "Experiment metadata + QC + signal URL", body = EpigenomeExperimentDetail),
        (status = 404, description = "Experiment not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn experiment(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<EpigenomeExperimentDetail>, ApiError> {
    let repository = state
        .epigenome_repository
        .as_ref()
        .ok_or_else(epigenome_unavailable)?;
    let id = parse_experiment_id(&experiment_id)?;
    let experiment = repository
        .experiment(&id)
        .ok_or(ServiceError::InvalidRequest(format!(
            "epigenome experiment not found: {experiment_id}"
        )))?;
    let peak_count = repository.peaks_for_experiment(&id).len();
    let signal_url = signal_url_for(&experiment, state.epigenome_base_path.as_deref());

    Ok(Json(EpigenomeExperimentDetail {
        experiment,
        peak_count,
        signal_url,
    }))
}

#[utoipa::path(
    get,
    path = "/v2/epigenome/peaks",
    operation_id = "epigenome_peaks",
    params(EpigenomePeaksQuery),
    responses(
        (status = 200, description = "Peaks overlapping the region", body = Vec<EpigenomePeakHit>),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn peaks(
    State(state): State<AppState>,
    Query(query): Query<EpigenomePeaksQuery>,
) -> Result<Json<Vec<EpigenomePeakHit>>, ApiError> {
    let Some(repository) = state.epigenome_repository.as_ref() else {
        return Ok(Json(Vec::new()));
    };

    let assembly_accession = parse_assembly(&query.assembly_accession)?;
    let region = ClosedRegion::from_str(&query.region)
        .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?
        .to_half_open()
        .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
    let target = query.target.as_deref().map(parse_target).transpose()?;
    let experiments = parse_experiment_ids(query.experiment_ids.as_deref())?;
    let limit = query.limit.unwrap_or(DEFAULT_PEAK_LIMIT);

    let hits = repository.peaks_in_region(&PeakRegionQuery {
        assembly_accession,
        region,
        experiments,
        assay: query.assay,
        target,
        limit: Some(limit),
    });

    Ok(Json(hits.into_iter().map(public_hit).collect()))
}

#[utoipa::path(
    get,
    path = "/v2/gene/id/{gene_id}/epigenome",
    operation_id = "gene_epigenome",
    params(
        ("gene_id" = String, Path, description = "Gene identifier"),
        GeneEpigenomeQuery,
    ),
    responses(
        (status = 200, description = "Experiments with peaks overlapping the gene body + flanks", body = EpigenomeGeneView),
        (status = 404, description = "Gene not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn gene_epigenome(
    State(state): State<AppState>,
    Path(gene_id): Path<String>,
    Query(query): Query<GeneEpigenomeQuery>,
) -> Result<Json<EpigenomeGeneView>, ApiError> {
    let (record, flank) =
        state
            .service
            .gene_flank_region(&gene_id, query.upstream_bp, query.downstream_bp)?;

    let assembly_accession = record.gene.assembly_accession.clone();
    let region = public_region_from(&flank);

    let Some(repository) = state.epigenome_repository.as_ref() else {
        return Ok(Json(EpigenomeGeneView {
            gene_id: record.gene.id.into_string(),
            assembly_accession: assembly_accession.into_string(),
            region,
            experiments: Vec::new(),
        }));
    };

    let target = query.target.as_deref().map(parse_target).transpose()?;
    let limit = query.limit.unwrap_or(DEFAULT_PEAK_LIMIT);

    let hits = repository.peaks_in_region(&PeakRegionQuery {
        assembly_accession: assembly_accession.clone(),
        region: flank,
        experiments: None,
        assay: query.assay,
        target,
        limit: Some(limit),
    });

    let mut by_experiment: BTreeMap<ExperimentId, Vec<Peak>> = BTreeMap::new();
    for PeakHit {
        experiment_id,
        peak,
    } in hits
    {
        by_experiment.entry(experiment_id).or_default().push(peak);
    }

    let mut experiments = Vec::with_capacity(by_experiment.len());
    for (experiment_id, peaks) in by_experiment {
        let Some(experiment) = repository.experiment(&experiment_id) else {
            continue;
        };
        let signal_url = signal_url_for(&experiment, state.epigenome_base_path.as_deref());
        experiments.push(EpigenomeExperimentWithPeaks {
            experiment: experiment_summary(experiment, signal_url),
            peaks: peaks.into_iter().map(PublicPeak::from).collect(),
        });
    }

    Ok(Json(EpigenomeGeneView {
        gene_id: record.gene.id.into_string(),
        assembly_accession: assembly_accession.into_string(),
        region,
        experiments,
    }))
}

fn experiment_summary(
    experiment: Experiment,
    signal_url: Option<String>,
) -> EpigenomeExperimentSummary {
    EpigenomeExperimentSummary {
        experiment_id: experiment.id.into_string(),
        assay: experiment.assay,
        target: experiment.target.map(Target::into_string),
        tissue: experiment.tissue,
        dev_stage: experiment.dev_stage,
        treatment: experiment.treatment,
        replicate: experiment.replicate,
        peak_kind: experiment.peak_kind,
        frip: experiment.qc.frip,
        signal_url,
    }
}

fn public_hit(hit: PeakHit) -> EpigenomePeakHit {
    EpigenomePeakHit {
        experiment_id: hit.experiment_id.into_string(),
        peak: hit.peak.into(),
    }
}

/// Project an internal half-open region to the API's 1-based closed shape.
fn public_region_from(region: &genome_domain::HalfOpenRegion) -> PublicRegion {
    PublicRegion {
        sequence_name: region.sequence_name.as_str().to_owned(),
        // half-open [start, end) → 1-based closed [start+1, end]
        start: region.start.get() + 1,
        end: region.end.get(),
    }
}

pub(crate) fn signal_url_for(experiment: &Experiment, base_path: Option<&str>) -> Option<String> {
    let signal_file = experiment.signal_file.as_deref()?;
    let base = base_path.unwrap_or("/epigenome/signal");
    Some(format!("{}/{}", base.trim_end_matches('/'), signal_file))
}

fn parse_assembly(value: &str) -> Result<AssemblyAccession, ApiError> {
    AssemblyAccession::new(value)
        .map_err(|error| ApiError::from(ServiceError::InvalidRequest(error.to_string())))
}

fn parse_target(value: &str) -> Result<Target, ApiError> {
    Target::new(value)
        .map_err(|error| ApiError::from(ServiceError::InvalidRequest(error.to_string())))
}

fn parse_experiment_id(value: &str) -> Result<ExperimentId, ApiError> {
    ExperimentId::new(value)
        .map_err(|error| ApiError::from(ServiceError::InvalidRequest(error.to_string())))
}

fn parse_experiment_ids(value: Option<&str>) -> Result<Option<Vec<ExperimentId>>, ApiError> {
    let Some(value) = value else { return Ok(None) };
    let ids = value
        .split(',')
        .map(str::trim)
        .filter(|piece| !piece.is_empty())
        .map(parse_experiment_id)
        .collect::<Result<Vec<_>, _>>()?;
    if ids.len() > MAX_EXPERIMENT_IDS {
        return Err(ApiError::from(ServiceError::InvalidRequest(format!(
            "experimentIds limit exceeded: {} > {MAX_EXPERIMENT_IDS}",
            ids.len()
        ))));
    }
    Ok(Some(ids))
}

fn epigenome_unavailable() -> ApiError {
    ApiError::from(ServiceError::InvalidRequest(
        "epigenome data is not loaded".to_owned(),
    ))
}

/// Build the JBrowse tracks for one assembly: a peak FeatureTrack and an
/// optional signal QuantitativeTrack per experiment in the assembly.
pub(crate) fn jbrowse_tracks(
    repository: &impl EpigenomeRepository,
    assembly_accession: &AssemblyAccession,
    base_url: Option<&str>,
    signal_base_path: Option<&str>,
) -> Vec<serde_json::Value> {
    let experiments = repository.experiments_for_assembly(assembly_accession);
    let mut tracks = Vec::with_capacity(experiments.len() * 2);
    for experiment in experiments {
        let track_id_base = format!("epigenome-{}", experiment.id.as_str());
        let name = experiment
            .target
            .as_ref()
            .map(|t| format!("{} {}", t.as_str(), experiment.assay.as_str()))
            .unwrap_or_else(|| {
                format!("{} ({})", experiment.id.as_str(), experiment.assay.as_str())
            });

        let peaks_uri = endpoint_url(
            base_url,
            &format!(
                "/v2/epigenome/peaks?assemblyAccession={}&experimentIds={}&region={{refName}}:{{start}}-{{end}}",
                assembly_accession.as_str(),
                experiment.id.as_str()
            ),
        );

        tracks.push(serde_json::json!({
            "type": "FeatureTrack",
            "trackId": format!("{track_id_base}-peaks"),
            "name": format!("{name} (peaks)"),
            "assemblyNames": [assembly_accession.as_str()],
            "category": ["Epigenome", experiment.assay.as_str()],
            "adapter": {
                "type": "PgpEpigenomePeaksAdapter",
                "experimentId": experiment.id.as_str(),
                "url": peaks_uri,
            },
        }));

        if let Some(signal_url) = signal_url_for(&experiment, signal_base_path) {
            let signal_uri = endpoint_url(base_url, &signal_url);
            tracks.push(serde_json::json!({
                "type": "QuantitativeTrack",
                "trackId": format!("{track_id_base}-signal"),
                "name": format!("{name} (signal)"),
                "assemblyNames": [assembly_accession.as_str()],
                "category": ["Epigenome", experiment.assay.as_str()],
                "adapter": {
                    "type": "BigWigAdapter",
                    "bigWigLocation": {
                        "uri": signal_uri,
                        "locationType": "UriLocation",
                    },
                },
            }));
        }
    }
    tracks
}

fn endpoint_url(base_url: Option<&str>, path: &str) -> String {
    match base_url
        .map(str::trim)
        .filter(|base_url| !base_url.is_empty())
    {
        Some(base_url) => format!("{}{}", base_url.trim_end_matches('/'), path),
        None => path.to_owned(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use epigenome_domain::{ExperimentQc, PeakKind};
    use genome_domain::{HalfOpenRegion, Position0, SequenceName, Strand};

    fn assembly() -> AssemblyAccession {
        AssemblyAccession::new("GCA_test").unwrap()
    }

    fn experiment(id: &str, signal_file: Option<&str>) -> Experiment {
        Experiment {
            id: ExperimentId::new(id).unwrap(),
            assay: Assay::ChipSeq,
            target: Some(Target::new("H3K4me3").unwrap()),
            antibody: None,
            assembly_accession: assembly(),
            geo_series: None,
            geo_sample: None,
            sra_runs: Vec::new(),
            tissue: None,
            dev_stage: None,
            treatment: None,
            replicate: None,
            pipeline: None,
            qvalue_cutoff: None,
            qc: ExperimentQc::default(),
            peak_kind: PeakKind::Narrow,
            signal_file: signal_file.map(str::to_owned),
            attributes: Default::default(),
        }
    }

    #[test]
    fn signal_url_uses_default_base_when_unset() {
        let exp = experiment("e1", Some("e1.bw"));
        assert_eq!(
            signal_url_for(&exp, None).as_deref(),
            Some("/epigenome/signal/e1.bw")
        );
    }

    #[test]
    fn signal_url_honours_custom_base_path() {
        let exp = experiment("e1", Some("e1.bw"));
        assert_eq!(
            signal_url_for(&exp, Some("/static/peaks")).as_deref(),
            Some("/static/peaks/e1.bw")
        );
    }

    #[test]
    fn signal_url_is_none_without_signal_file() {
        let exp = experiment("e1", None);
        assert!(signal_url_for(&exp, None).is_none());
    }

    #[test]
    fn public_peak_projects_half_open_to_one_based_closed() {
        let region = HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(100),
            Position0::new(200),
        )
        .unwrap();
        let peak = Peak {
            region,
            name: "p1".to_owned(),
            score: 500,
            strand: Strand::Unknown,
            signal_value: 10.0,
            p_value: 20.0,
            q_value: 15.0,
            summit_offset: Some(50),
        };
        let public = PublicPeak::from(peak);
        assert_eq!(public.start, 101);
        assert_eq!(public.end, 200);
    }

    #[test]
    fn parse_experiment_ids_accepts_comma_list() {
        let ids = parse_experiment_ids(Some("exp_1,exp_2 , exp_3"))
            .unwrap()
            .unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn parse_experiment_ids_returns_none_for_unset() {
        assert!(parse_experiment_ids(None).unwrap().is_none());
        assert!(parse_experiment_ids(Some("")).unwrap().unwrap().is_empty());
    }

    #[test]
    fn parse_experiment_ids_accepts_up_to_max_and_rejects_above() {
        // Pins the `> MAX_EXPERIMENT_IDS` boundary so swapping to `>=` (which
        // would reject the at-limit case) or `==` is caught.
        let at_limit = (0..MAX_EXPERIMENT_IDS)
            .map(|i| format!("e{i}"))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            parse_experiment_ids(Some(&at_limit))
                .unwrap()
                .unwrap()
                .len(),
            MAX_EXPERIMENT_IDS
        );

        let above_limit = (0..=MAX_EXPERIMENT_IDS)
            .map(|i| format!("e{i}"))
            .collect::<Vec<_>>()
            .join(",");
        assert!(parse_experiment_ids(Some(&above_limit)).is_err());
    }

    #[test]
    fn endpoint_url_returns_path_when_base_is_empty_or_unset() {
        // Pins the `!base_url.is_empty()` guard so deleting the `!` (which
        // would prepend an empty base + reformat) is caught.
        assert_eq!(endpoint_url(None, "/v2/foo"), "/v2/foo");
        assert_eq!(endpoint_url(Some(""), "/v2/foo"), "/v2/foo");
        assert_eq!(endpoint_url(Some("   "), "/v2/foo"), "/v2/foo");
        assert_eq!(
            endpoint_url(Some("http://api.test/"), "/v2/foo"),
            "http://api.test/v2/foo"
        );
    }

    #[test]
    fn default_upstream_bp_matches_documented_default() {
        // Pins the constant so silent edits to the promoter-window default
        // are caught.
        assert_eq!(default_upstream_bp(), 2_000);
    }

    fn make_region(start: u64, end: u64) -> genome_domain::HalfOpenRegion {
        HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(start),
            Position0::new(end),
        )
        .unwrap()
    }

    #[test]
    fn public_region_from_half_open_shifts_start_by_one() {
        // half-open [99, 200) → 1-based closed [100, 200]. Pinning this
        // boundary kills arithmetic mutations on the `+ 1` in
        // public_region_from (called by gene_epigenome).
        let public = public_region_from(&make_region(99, 200));
        assert_eq!(public.start, 100);
        assert_eq!(public.end, 200);
        assert_eq!(public.sequence_name, "chr1");
    }
}
