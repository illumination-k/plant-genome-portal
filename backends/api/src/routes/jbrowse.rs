use axum::{
    Json,
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
};
use genome_core::{Gene, Sequence, Strand};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::{IntoParams, ToSchema};

use crate::{ApiError, AppService, AppState};
use service::ServiceError;

#[utoipa::path(
    get,
    path = "/jbrowse/config",
    params(JBrowseConfigQuery),
    responses(
        (status = 200, description = "Default JBrowse launch config", body = JBrowseRootConfig),
        (status = 404, description = "Assembly not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn default_config(
    State(state): State<AppState>,
    Query(query): Query<JBrowseConfigQuery>,
) -> Result<Json<JBrowseRootConfig>, ApiError> {
    let accession = state.default_assembly_accession.clone();
    config_for_accession(&state.service, &accession, query.base_url.as_deref()).map(Json)
}

#[utoipa::path(
    get,
    path = "/jbrowse/config/{accession}",
    params(
        ("accession" = String, Path, description = "Assembly accession"),
        JBrowseConfigQuery,
    ),
    responses(
        (status = 200, description = "JBrowse launch config", body = JBrowseRootConfig),
        (status = 404, description = "Assembly not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn config(
    State(state): State<AppState>,
    Path(accession): Path<String>,
    Query(query): Query<JBrowseConfigQuery>,
) -> Result<Json<JBrowseRootConfig>, ApiError> {
    config_for_accession(&state.service, &accession, query.base_url.as_deref()).map(Json)
}

#[utoipa::path(
    get,
    path = "/jbrowse/assemblies/{accession}/chrom.sizes",
    params(("accession" = String, Path, description = "Assembly accession")),
    responses(
        (status = 200, description = "UCSC chrom.sizes compatible sequence sizes", content_type = "text/plain", body = String),
        (status = 404, description = "Assembly not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn chrom_sizes(
    State(state): State<AppState>,
    Path(accession): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let sequences = state.service.sequences_for_assembly(&accession)?;
    Ok((
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        chrom_sizes_body(sequences),
    ))
}

#[utoipa::path(
    get,
    path = "/jbrowse/assemblies/{accession}/features",
    params(
        ("accession" = String, Path, description = "Assembly accession"),
        JBrowseFeaturesQuery,
    ),
    responses(
        (status = 200, description = "Features for a JBrowse custom adapter", body = Vec<JBrowseFeature>),
        (status = 404, description = "Assembly not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn features(
    State(state): State<AppState>,
    Path(accession): Path<String>,
    Query(query): Query<JBrowseFeaturesQuery>,
) -> Result<Json<Vec<JBrowseFeature>>, ApiError> {
    if query.start >= query.end {
        return Err(ServiceError::InvalidRequest("start must be less than end".to_owned()).into());
    }

    let region = format!("{}:{}-{}", query.ref_name, query.start + 1, query.end);
    let features = state
        .service
        .features_in_region(&accession, &region)?
        .into_iter()
        .map(JBrowseFeature::from)
        .collect();
    Ok(Json(features))
}

fn config_for_accession(
    service: &AppService,
    accession: &str,
    base_url: Option<&str>,
) -> Result<JBrowseRootConfig, ApiError> {
    let assembly = service.assembly(accession)?;
    let sequences = service.sequences_for_assembly(accession)?;
    Ok(build_config(&assembly, &sequences, base_url))
}

pub(crate) fn build_config(
    assembly: &genome_core::Assembly,
    sequences: &[Sequence],
    base_url: Option<&str>,
) -> JBrowseRootConfig {
    let accession = assembly.accession.as_str();
    let initial_ref = sequences
        .iter()
        .min_by(|left, right| left.name.as_str().cmp(right.name.as_str()))
        .map(|sequence| sequence.name.as_str())
        .unwrap_or("chr1");
    let initial_end = sequences
        .iter()
        .find(|sequence| sequence.name.as_str() == initial_ref)
        .map(|sequence| sequence.length.min(100_000))
        .unwrap_or(100_000);
    let loc = format!("{initial_ref}:1..{initial_end}");
    let chrom_sizes_url = endpoint_url(
        base_url,
        &format!("/jbrowse/assemblies/{accession}/chrom.sizes"),
    );
    let features_url = endpoint_url(
        base_url,
        &format!("/jbrowse/assemblies/{accession}/features"),
    );

    JBrowseRootConfig {
        assemblies: vec![JBrowseAssembly {
            name: accession.to_owned(),
            aliases: vec![assembly.name.clone()],
            sequence: JBrowseSequenceTrack {
                track_type: "ReferenceSequenceTrack".to_owned(),
                track_id: format!("{accession}-ReferenceSequenceTrack"),
                adapter: JBrowseChromSizesAdapter {
                    adapter_type: "ChromSizesAdapter".to_owned(),
                    chrom_sizes_location: JBrowseUriLocation {
                        uri: chrom_sizes_url.clone(),
                        location_type: "UriLocation".to_owned(),
                    },
                },
                rendering: JBrowseRendering {
                    rendering_type: "DivSequenceRenderer".to_owned(),
                },
            },
        }],
        tracks: Vec::new(),
        default_session: JBrowseDefaultSession {
            name: format!("{} genome browser", assembly.name),
            view: JBrowseDefaultView {
                id: "linearGenomeView".to_owned(),
                view_type: "LinearGenomeView".to_owned(),
                init: JBrowseDefaultViewInit {
                    assembly: accession.to_owned(),
                    loc,
                    tracks: Vec::new(),
                },
            },
        },
        plant_genome_portal: JBrowsePortalConfig {
            assembly_accession: accession.to_owned(),
            chrom_sizes_url,
            features_url,
            features_url_template: endpoint_url(
                base_url,
                &format!(
                    "/jbrowse/assemblies/{accession}/features?refName={{refName}}&start={{start}}&end={{end}}"
                ),
            ),
            sequence_url_template: endpoint_url(
                base_url,
                "/sequence/{checksum}?start={start}&end={end}",
            ),
        },
    }
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

pub(crate) fn chrom_sizes_body(mut sequences: Vec<Sequence>) -> String {
    sequences.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
    sequences
        .into_iter()
        .map(|sequence| format!("{}\t{}", sequence.name.as_str(), sequence.length))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub(crate) struct JBrowseConfigQuery {
    #[serde(default, alias = "baseUrl")]
    base_url: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub(crate) struct JBrowseFeaturesQuery {
    #[serde(alias = "refName")]
    ref_name: String,
    start: u64,
    end: u64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JBrowseRootConfig {
    assemblies: Vec<JBrowseAssembly>,
    tracks: Vec<JBrowseTrack>,
    default_session: JBrowseDefaultSession,
    plant_genome_portal: JBrowsePortalConfig,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JBrowseAssembly {
    name: String,
    aliases: Vec<String>,
    sequence: JBrowseSequenceTrack,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JBrowseSequenceTrack {
    #[serde(rename = "type")]
    track_type: String,
    track_id: String,
    adapter: JBrowseChromSizesAdapter,
    rendering: JBrowseRendering,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JBrowseChromSizesAdapter {
    #[serde(rename = "type")]
    adapter_type: String,
    chrom_sizes_location: JBrowseUriLocation,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JBrowseUriLocation {
    uri: String,
    location_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JBrowseRendering {
    #[serde(rename = "type")]
    rendering_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct JBrowseTrack {}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JBrowseDefaultSession {
    name: String,
    view: JBrowseDefaultView,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JBrowseDefaultView {
    id: String,
    #[serde(rename = "type")]
    view_type: String,
    init: JBrowseDefaultViewInit,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JBrowseDefaultViewInit {
    assembly: String,
    loc: String,
    tracks: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JBrowsePortalConfig {
    assembly_accession: String,
    chrom_sizes_url: String,
    features_url: String,
    features_url_template: String,
    sequence_url_template: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JBrowseFeature {
    unique_id: String,
    ref_name: String,
    start: u64,
    end: u64,
    name: String,
    #[serde(rename = "type")]
    feature_type: String,
    strand: i8,
    attributes: BTreeMap<String, String>,
}

impl From<Gene> for JBrowseFeature {
    fn from(gene: Gene) -> Self {
        let name = gene
            .symbol
            .clone()
            .or_else(|| gene.locus_tag.clone())
            .unwrap_or_else(|| gene.id.as_str().to_owned());
        Self {
            unique_id: gene.id.as_str().to_owned(),
            ref_name: gene.sequence_name.as_str().to_owned(),
            start: gene.region.start.get(),
            end: gene.region.end.get(),
            name,
            feature_type: gene.feature_type,
            strand: match gene.strand {
                Strand::Forward => 1,
                Strand::Reverse => -1,
                Strand::Unknown => 0,
            },
            attributes: gene.attributes,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use genome_core::{
        Assembly, AssemblyAccession, AssemblySource, HalfOpenRegion, Position0, SequenceName, TaxId,
    };

    #[test]
    fn jbrowse_config_uses_chrom_sizes_adapter_and_default_location() {
        let assembly = Assembly {
            accession: AssemblyAccession::new("GCA_test").unwrap(),
            tax_id: TaxId::new(3197),
            name: "TestAssembly".to_owned(),
            source: AssemblySource::Local,
            refget_checksum: None,
        };
        let sequences = vec![Sequence {
            name: SequenceName::new("chr2").unwrap(),
            assembly_accession: assembly.accession.clone(),
            length: 50_000,
            refget_checksum: "checksum".to_owned(),
        }];

        let config = build_config(&assembly, &sequences, Some("http://api.test/"));

        assert_eq!(config.assemblies[0].name, "GCA_test");
        assert_eq!(
            config.assemblies[0].sequence.adapter.adapter_type,
            "ChromSizesAdapter"
        );
        assert_eq!(
            config.assemblies[0]
                .sequence
                .adapter
                .chrom_sizes_location
                .uri,
            "http://api.test/jbrowse/assemblies/GCA_test/chrom.sizes"
        );
        assert_eq!(config.default_session.view.init.loc, "chr2:1..50000");
        assert_eq!(
            config.plant_genome_portal.features_url_template,
            "http://api.test/jbrowse/assemblies/GCA_test/features?refName={refName}&start={start}&end={end}"
        );
    }

    #[test]
    fn chrom_sizes_body_is_sorted_and_tab_delimited() {
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        let sequences = vec![
            Sequence {
                name: SequenceName::new("chr2").unwrap(),
                assembly_accession: accession.clone(),
                length: 20,
                refget_checksum: "checksum2".to_owned(),
            },
            Sequence {
                name: SequenceName::new("chr1").unwrap(),
                assembly_accession: accession,
                length: 10,
                refget_checksum: "checksum1".to_owned(),
            },
        ];

        assert_eq!(chrom_sizes_body(sequences), "chr1\t10\nchr2\t20\n");
    }

    #[test]
    fn gene_converts_to_jbrowse_feature_coordinates() {
        let gene = Gene {
            id: genome_core::GeneId::new("gene1").unwrap(),
            assembly_accession: AssemblyAccession::new("GCA_test").unwrap(),
            symbol: Some("SYMBOL1".to_owned()),
            locus_tag: None,
            sequence_name: SequenceName::new("chr1").unwrap(),
            region: HalfOpenRegion::new(
                SequenceName::new("chr1").unwrap(),
                Position0::new(9),
                Position0::new(20),
            )
            .unwrap(),
            strand: Strand::Reverse,
            feature_type: "gene".to_owned(),
            annotations: Vec::new(),
            attributes: BTreeMap::new(),
        };

        let feature = JBrowseFeature::from(gene);

        assert_eq!(feature.unique_id, "gene1");
        assert_eq!(feature.name, "SYMBOL1");
        assert_eq!(feature.ref_name, "chr1");
        assert_eq!(feature.start, 9);
        assert_eq!(feature.end, 20);
        assert_eq!(feature.strand, -1);
    }
}
