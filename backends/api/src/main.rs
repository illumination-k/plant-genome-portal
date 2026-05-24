use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use clap::Parser;
use genome_core::{GeneSearch, TaxId};
use serde::{Deserialize, Serialize};
use service::{GenomeService, ServiceError};
use std::net::SocketAddr;
use std::path::PathBuf;
use storage::{FastaReference, FileGenomeRepository};
use tracing_subscriber::EnvFilter;
use utoipa::{IntoParams, OpenApi, ToSchema};

type AppService = GenomeService<FileGenomeRepository>;

#[derive(Clone)]
struct AppState {
    service: AppService,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = Config::parse();
    let repository = FileGenomeRepository::from_snapshot_path(&config.snapshot)?;
    let reference = match (&config.fasta, config.no_fasta) {
        (_, true) | (None, false) => None,
        (Some(path), false) => Some(FastaReference::from_path(path)?),
    };
    let service = GenomeService::new(repository, reference);

    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(
        bind = %config.bind,
        snapshot = %config.snapshot.display(),
        "starting api"
    );
    axum::serve(listener, router(service)).await?;

    Ok(())
}

fn router(service: AppService) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/openapi.json", get(openapi_json))
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
        .with_state(AppState { service })
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
    #[arg(long, default_value = "127.0.0.1:3000")]
    bind: SocketAddr,
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long, conflicts_with = "no_fasta")]
    fasta: Option<PathBuf>,
    #[arg(long)]
    no_fasta: bool,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
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
        RefgetQuery,
        RefgetServiceInfo,
        TaxonResponse,
        genome_core::Assembly,
        genome_core::AssemblyAccession,
        genome_core::AssemblySource,
        genome_core::ClosedRegion,
        genome_core::Exon,
        genome_core::Gene,
        genome_core::GeneId,
        genome_core::GeneRecord,
        genome_core::HalfOpenRegion,
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

#[derive(Debug)]
struct ApiError(ServiceError);

impl From<ServiceError> for ApiError {
    fn from(error: ServiceError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
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
