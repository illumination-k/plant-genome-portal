use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use clap::{Parser, Subcommand};
use genome_core::{Gene, GeneSearch, Sequence, Strand, TaxId};
use serde::{Deserialize, Serialize};
use service::{GenomeService, ServiceError};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::{Path as FsPath, PathBuf};
use storage::{FastaReference, FileGenomeRepository};
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;
use utoipa::{IntoParams, OpenApi, ToSchema};

type AppService = GenomeService<FileGenomeRepository>;

#[derive(Clone)]
struct AppState {
    service: AppService,
    default_assembly_accession: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = Config::parse();
    if let Some(command) = config.command {
        match command {
            Command::Openapi { output } => write_openapi_schema(output.as_deref())?,
        }
        return Ok(());
    }

    let snapshot = config.snapshot.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "--snapshot is required when running the API server",
        )
    })?;
    let repository = FileGenomeRepository::from_snapshot_path(&snapshot)?;
    let default_assembly_accession = repository.default_assembly_accession().into_string();
    let reference = match (&config.fasta, config.no_fasta) {
        (_, true) | (None, false) => None,
        (Some(path), false) => Some(FastaReference::from_path(path)?),
    };
    let service = GenomeService::new(repository, reference);

    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(
        bind = %config.bind,
        snapshot = %snapshot.display(),
        "starting api"
    );
    axum::serve(listener, router(service, default_assembly_accession)).await?;

    Ok(())
}

fn write_openapi_schema(output: Option<&FsPath>) -> Result<(), Box<dyn std::error::Error>> {
    let schema = ApiDoc::openapi();

    match output {
        Some(path) => {
            if let Some(parent) = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                std::fs::create_dir_all(parent)?;
            }
            serde_json::to_writer_pretty(File::create(path)?, &schema)?;
        }
        None => {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            serde_json::to_writer_pretty(&mut stdout, &schema)?;
            stdout.write_all(b"\n")?;
        }
    }

    Ok(())
}

fn router(service: AppService, default_assembly_accession: String) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/openapi.json", get(openapi_json))
        .route("/jbrowse/config", get(jbrowse_default_config))
        .route("/jbrowse/config/{accession}", get(jbrowse_config))
        .route(
            "/jbrowse/assemblies/{accession}/chrom.sizes",
            get(jbrowse_chrom_sizes),
        )
        .route(
            "/jbrowse/assemblies/{accession}/features",
            get(jbrowse_features),
        )
        .route("/v2/genome/accession/{accession}", get(assembly))
        .route(
            "/v2/genome/accession/{accession}/sequences",
            get(assembly_sequences),
        )
        .route("/v2/genome/taxon/{tax_id}", get(taxon))
        .route("/v2/gene/id/{gene_id}", get(gene))
        .route("/v2/gene/search", get(gene_search))
        .route(
            "/v2/genome/accession/{accession}/region/{region}/features",
            get(region_features),
        )
        .route("/sequence/service-info", get(refget_service_info))
        .route("/sequence/{checksum}", get(refget_sequence))
        .layer(CorsLayer::permissive())
        .with_state(AppState {
            service,
            default_assembly_accession,
        })
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Health check", body = HealthResponse))
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { ok: true })
}

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[utoipa::path(
    get,
    path = "/jbrowse/config",
    params(JBrowseConfigQuery),
    responses(
        (status = 200, description = "Default JBrowse launch config", body = JBrowseRootConfig),
        (status = 404, description = "Assembly not found", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
async fn jbrowse_default_config(
    State(state): State<AppState>,
    Query(query): Query<JBrowseConfigQuery>,
) -> Result<Json<JBrowseRootConfig>, ApiError> {
    let accession = state.default_assembly_accession.clone();
    jbrowse_config_for_accession(&state.service, &accession, query.base_url.as_deref()).map(Json)
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
        (status = 404, description = "Assembly not found", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
async fn jbrowse_config(
    State(state): State<AppState>,
    Path(accession): Path<String>,
    Query(query): Query<JBrowseConfigQuery>,
) -> Result<Json<JBrowseRootConfig>, ApiError> {
    jbrowse_config_for_accession(&state.service, &accession, query.base_url.as_deref()).map(Json)
}

#[utoipa::path(
    get,
    path = "/jbrowse/assemblies/{accession}/chrom.sizes",
    params(("accession" = String, Path, description = "Assembly accession")),
    responses(
        (status = 200, description = "UCSC chrom.sizes compatible sequence sizes", content_type = "text/plain", body = String),
        (status = 404, description = "Assembly not found", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
async fn jbrowse_chrom_sizes(
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
        (status = 404, description = "Assembly not found", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
async fn jbrowse_features(
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

#[utoipa::path(
    get,
    path = "/v2/genome/accession/{accession}",
    params(("accession" = String, Path, description = "Assembly accession")),
    responses(
        (status = 200, description = "Assembly metadata", body = genome_core::Assembly),
        (status = 404, description = "Assembly not found", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
async fn assembly(
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
        (status = 404, description = "Assembly not found", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
async fn assembly_sequences(
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
        (status = 404, description = "Taxon not found", body = ErrorResponse),
    )
)]
async fn taxon(
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
    path = "/v2/gene/id/{gene_id}",
    params(("gene_id" = String, Path, description = "Gene identifier")),
    responses(
        (status = 200, description = "Gene detail", body = genome_core::GeneRecord),
        (status = 404, description = "Gene not found", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
async fn gene(
    State(state): State<AppState>,
    Path(gene_id): Path<String>,
) -> Result<Json<genome_core::GeneRecord>, ApiError> {
    Ok(Json(state.service.gene(&gene_id)?))
}

#[utoipa::path(
    get,
    path = "/v2/gene/search",
    params(GeneSearchQuery),
    responses(
        (status = 200, description = "Matching genes", body = Vec<genome_core::Gene>),
    )
)]
async fn gene_search(
    State(state): State<AppState>,
    Query(query): Query<GeneSearchQuery>,
) -> Json<Vec<genome_core::Gene>> {
    Json(state.service.search_genes(query.into_search()))
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
        (status = 404, description = "Assembly not found", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
async fn region_features(
    State(state): State<AppState>,
    Path((accession, region)): Path<(String, String)>,
) -> Result<Json<Vec<genome_core::Gene>>, ApiError> {
    Ok(Json(state.service.features_in_region(&accession, &region)?))
}

#[utoipa::path(
    get,
    path = "/sequence/service-info",
    responses((status = 200, description = "refget service info", body = RefgetServiceInfo))
)]
async fn refget_service_info() -> Json<RefgetServiceInfo> {
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
    params(
        ("checksum" = String, Path, description = "refget checksum"),
        RefgetQuery,
    ),
    responses(
        (status = 200, description = "Reference sequence", body = String),
        (status = 404, description = "Sequence not found", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    )
)]
async fn refget_sequence(
    State(state): State<AppState>,
    Path(checksum): Path<String>,
    Query(query): Query<RefgetQuery>,
) -> Result<String, ApiError> {
    Ok(state
        .service
        .refget_sequence(&checksum, query.start, query.end)?)
}

#[derive(Debug, Parser)]
#[command(name = "api")]
#[command(about = "Plant Genome Portal HTTP API")]
struct Config {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(long, default_value = "127.0.0.1:3000")]
    bind: SocketAddr,
    #[arg(long)]
    snapshot: Option<PathBuf>,
    #[arg(long, conflicts_with = "no_fasta")]
    fasta: Option<PathBuf>,
    #[arg(long)]
    no_fasta: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Write the OpenAPI schema without starting the API server.
    Openapi {
        /// Path to write the schema JSON. Writes to stdout when omitted.
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        jbrowse_default_config,
        jbrowse_config,
        jbrowse_chrom_sizes,
        jbrowse_features,
        assembly,
        assembly_sequences,
        taxon,
        gene,
        gene_search,
        region_features,
        refget_service_info,
        refget_sequence,
    ),
    components(schemas(
        ErrorResponse,
        GeneSearchQuery,
        HealthResponse,
        JBrowseAssembly,
        JBrowseChromSizesAdapter,
        JBrowseConfigQuery,
        JBrowseDefaultSession,
        JBrowseDefaultView,
        JBrowseDefaultViewInit,
        JBrowseFeature,
        JBrowseFeaturesQuery,
        JBrowsePortalConfig,
        JBrowseRendering,
        JBrowseRootConfig,
        JBrowseSequenceTrack,
        JBrowseTrack,
        JBrowseUriLocation,
        RefgetQuery,
        RefgetServiceInfo,
        TaxonResponse,
        genome_core::AnnotationEvidence,
        genome_core::AnnotationSource,
        genome_core::Assembly,
        genome_core::AssemblyAccession,
        genome_core::AssemblySource,
        genome_core::ClosedRegion,
        genome_core::Exon,
        genome_core::FunctionalAnnotation,
        genome_core::Gene,
        genome_core::GeneId,
        genome_core::GeneRecord,
        genome_core::GoNamespace,
        genome_core::GoTermAnnotation,
        genome_core::GoTermId,
        genome_core::HalfOpenRegion,
        genome_core::InterProAnnotation,
        genome_core::InterProId,
        genome_core::KeggAnnotation,
        genome_core::KeggEntryId,
        genome_core::KeggEntryKind,
        genome_core::KogAnnotation,
        genome_core::KogEntryId,
        genome_core::NcbiFamAccession,
        genome_core::NcbiFamAnnotation,
        genome_core::PfamAccession,
        genome_core::PfamAnnotation,
        genome_core::Position0,
        genome_core::Position1,
        genome_core::Sequence,
        genome_core::SequenceName,
        genome_core::Strand,
        genome_core::TaxId,
        genome_core::Taxon,
        genome_core::Transcript,
        genome_core::TranscriptId,
    )),
    tags((name = "plant-genome-portal", description = "Plant Genome Portal API"))
)]
struct ApiDoc;

#[derive(Debug, Serialize, ToSchema)]
struct HealthResponse {
    ok: bool,
}

#[derive(Debug, Serialize, ToSchema)]
struct TaxonResponse {
    taxon: genome_core::Taxon,
    assemblies: Vec<genome_core::Assembly>,
}

fn jbrowse_config_for_accession(
    service: &AppService,
    accession: &str,
    base_url: Option<&str>,
) -> Result<JBrowseRootConfig, ApiError> {
    let assembly = service.assembly(accession)?;
    let sequences = service.sequences_for_assembly(accession)?;
    Ok(build_jbrowse_config(&assembly, &sequences, base_url))
}

fn build_jbrowse_config(
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

fn chrom_sizes_body(mut sequences: Vec<Sequence>) -> String {
    sequences.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
    sequences
        .into_iter()
        .map(|sequence| format!("{}\t{}", sequence.name.as_str(), sequence.length))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
struct GeneSearchQuery {
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

#[derive(Debug, Serialize, ToSchema)]
struct RefgetServiceInfo {
    id: String,
    name: String,
    circular_supported: bool,
    subsequence_limit: Option<u64>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
struct RefgetQuery {
    start: Option<u64>,
    end: Option<u64>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
struct JBrowseConfigQuery {
    #[serde(default, alias = "baseUrl")]
    base_url: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
struct JBrowseFeaturesQuery {
    #[serde(alias = "refName")]
    ref_name: String,
    start: u64,
    end: u64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JBrowseRootConfig {
    assemblies: Vec<JBrowseAssembly>,
    tracks: Vec<JBrowseTrack>,
    default_session: JBrowseDefaultSession,
    plant_genome_portal: JBrowsePortalConfig,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JBrowseAssembly {
    name: String,
    aliases: Vec<String>,
    sequence: JBrowseSequenceTrack,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JBrowseSequenceTrack {
    #[serde(rename = "type")]
    track_type: String,
    track_id: String,
    adapter: JBrowseChromSizesAdapter,
    rendering: JBrowseRendering,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JBrowseChromSizesAdapter {
    #[serde(rename = "type")]
    adapter_type: String,
    chrom_sizes_location: JBrowseUriLocation,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JBrowseUriLocation {
    uri: String,
    location_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JBrowseRendering {
    #[serde(rename = "type")]
    rendering_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct JBrowseTrack {}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JBrowseDefaultSession {
    name: String,
    view: JBrowseDefaultView,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JBrowseDefaultView {
    id: String,
    #[serde(rename = "type")]
    view_type: String,
    init: JBrowseDefaultViewInit,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JBrowseDefaultViewInit {
    assembly: String,
    loc: String,
    tracks: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JBrowsePortalConfig {
    assembly_accession: String,
    chrom_sizes_url: String,
    features_url: String,
    features_url_template: String,
    sequence_url_template: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JBrowseFeature {
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

#[derive(Debug)]
struct ApiError(ServiceError);

impl From<ServiceError> for ApiError {
    fn from(error: ServiceError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            ServiceError::TaxonNotFound(_)
            | ServiceError::AssemblyNotFound(_)
            | ServiceError::GeneNotFound(_)
            | ServiceError::SequenceNotFound(_) => StatusCode::NOT_FOUND,
            ServiceError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
        };

        (
            status,
            Json(ErrorResponse {
                error: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

#[derive(Debug, Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use genome_core::{
        Assembly, AssemblyAccession, AssemblySource, HalfOpenRegion, Position0, SequenceName,
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

        let config = build_jbrowse_config(&assembly, &sequences, Some("http://api.test/"));

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
