mod protein;
mod refget;
mod routes;
mod sequence;
#[cfg(test)]
mod test_support;

use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use clap::{Parser, Subcommand};
use epigenome_store::FileEpigenomeRepository;
use expression_store::FileExpressionRepository;
use genome_service::{
    GeneKeggOrthologyEntry, GeneKeggView, GenomeService, JobManagerError, KeggGeneSummary,
    KeggPathwayDetail, KeggPathwayKoEntry, KeggPathwaySummary, ServiceError,
};
use genome_store::{FastaReference, FileGenomeRepository};
use routes::blast::{BlastJobManager, BlastMethod, BlastWorkerCommand, BlastpJobManager};
use serde::Serialize;
use std::fs::File;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::{Path as FsPath, PathBuf};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;
use utoipa::{OpenApi, ToSchema};

pub(crate) type AppService = GenomeService<FileGenomeRepository>;
pub(crate) type AppExpressionRepository = FileExpressionRepository;
pub(crate) type AppEpigenomeRepository = FileEpigenomeRepository;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) service: AppService,
    pub(crate) expression_repository: Option<AppExpressionRepository>,
    pub(crate) epigenome_repository: Option<AppEpigenomeRepository>,
    /// URL prefix where bigWig signal files are served, e.g.
    /// `/epigenome/signal` (default) or `https://cdn.example/epigenome`.
    pub(crate) epigenome_base_path: Option<String>,
    pub(crate) default_assembly_accession: String,
    pub(crate) blast_jobs: Option<BlastJobManager>,
    pub(crate) blastp_jobs: Option<BlastpJobManager>,
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
        (Some(path), false) => {
            let mut reference = FastaReference::from_path(path)?;
            if let Some(protein_path) = &config.protein_fasta {
                reference.extend_from_path(protein_path)?;
            }
            Some(reference)
        }
    };
    let service = GenomeService::new(repository, reference);
    let expression_repository = config
        .expression_snapshot
        .as_deref()
        .map(FileExpressionRepository::from_snapshot_path)
        .transpose()?;
    let epigenome_repository = config
        .epigenome_snapshot
        .as_deref()
        .map(FileEpigenomeRepository::from_snapshot_path)
        .transpose()?;
    let epigenome_signal_root = config.epigenome_signal_root.clone();
    let epigenome_base_path = config.epigenome_base_path.clone();
    let blast_jobs = config.blast_db_prefix.clone().map(|blast_db_prefix| {
        routes::blast::BlastJobManager::new(BlastWorkerCommand {
            worker_bin: config.blast_worker_bin.clone(),
            blast_db_prefix,
            work_dir: config.blast_work_dir.clone(),
            program: config.blastn.clone(),
            snapshot: Some(snapshot.clone()),
            method: BlastMethod::Blastn,
        })
    });
    let blastp_jobs = config.blastp_db_prefix.clone().map(|blast_db_prefix| {
        routes::blast::BlastpJobManager::new(BlastWorkerCommand {
            worker_bin: config.blast_worker_bin.clone(),
            blast_db_prefix,
            work_dir: config.blast_work_dir.clone(),
            program: config.blastp.clone(),
            snapshot: Some(snapshot.clone()),
            method: BlastMethod::Blastp,
        })
    });

    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(
        bind = %config.bind,
        snapshot = %snapshot.display(),
        "starting api"
    );
    axum::serve(
        listener,
        router(RouterBuild {
            service,
            expression_repository,
            epigenome_repository,
            epigenome_base_path,
            epigenome_signal_root,
            default_assembly_accession,
            blast_jobs,
            blastp_jobs,
        }),
    )
    .await?;

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

struct RouterBuild {
    service: AppService,
    expression_repository: Option<AppExpressionRepository>,
    epigenome_repository: Option<AppEpigenomeRepository>,
    epigenome_base_path: Option<String>,
    epigenome_signal_root: Option<PathBuf>,
    default_assembly_accession: String,
    blast_jobs: Option<BlastJobManager>,
    blastp_jobs: Option<BlastpJobManager>,
}

fn router(build: RouterBuild) -> Router {
    let RouterBuild {
        service,
        expression_repository,
        epigenome_repository,
        epigenome_base_path,
        epigenome_signal_root,
        default_assembly_accession,
        blast_jobs,
        blastp_jobs,
    } = build;
    let mut router = Router::new()
        .route("/health", get(health))
        .route("/openapi.json", get(openapi_json))
        .route("/jbrowse/config", get(routes::jbrowse::default_config))
        .route("/jbrowse/config/{accession}", get(routes::jbrowse::config))
        .route(
            "/jbrowse/assemblies/{accession}/chrom.sizes",
            get(routes::jbrowse::chrom_sizes),
        )
        .route(
            "/jbrowse/assemblies/{accession}/features",
            get(routes::jbrowse::features),
        )
        .route(
            "/v2/genome/accession/{accession}",
            get(routes::genome::assembly),
        )
        .route(
            "/v2/genome/accession/{accession}/sequences",
            get(routes::genome::assembly_sequences),
        )
        .route("/v2/genome/taxon/{tax_id}", get(routes::genome::taxon))
        .route("/v2/gene/id/{gene_id}", get(routes::gene::gene))
        .route("/v2/gene/id/{gene_id}/kegg", get(routes::gene::gene_kegg))
        .route(
            "/v2/gene/id/{gene_id}/orthogroups",
            get(routes::gene::gene_orthogroups),
        )
        .route(
            "/v2/orthogroup/{orthogroup_id}",
            get(routes::gene::orthogroup),
        )
        .route("/v2/kegg/pathways", get(routes::gene::kegg_pathways))
        .route(
            "/v2/kegg/pathway/{pathway_id}",
            get(routes::gene::kegg_pathway),
        )
        .route(
            "/v2/gene/id/{gene_id}/expression",
            get(routes::expression::gene_expression),
        )
        .route(
            "/v2/expression/clustergram",
            get(routes::expression::clustergram),
        )
        .route(
            "/v2/analysis/enrichment",
            post(routes::enrichment::analysis),
        )
        .route("/v2/gene/search", get(routes::gene::gene_search))
        .route(
            "/v2/transcript/id/{transcript_id}/protein",
            get(protein::sequence),
        )
        .route(
            "/v2/tools/blastn/jobs",
            post(routes::blast::create_blastn_job),
        )
        .route(
            "/v2/tools/blastn/jobs/{job_id}",
            get(routes::blast::blastn_job),
        )
        .route(
            "/v2/tools/blastp/jobs",
            post(routes::blast::create_blastp_job),
        )
        .route(
            "/v2/tools/blastp/jobs/{job_id}",
            get(routes::blast::blastp_job),
        )
        .route(
            "/v2/genome/accession/{accession}/sequence/{sequence_name}",
            get(sequence::segments),
        )
        .route(
            "/v2/genome/accession/{accession}/region/{region}/features",
            get(routes::genome::region_features),
        )
        .route("/sequence/service-info", get(refget::service_info))
        .route("/sequence/{checksum}", get(refget::sequence))
        .route(
            "/v2/epigenome/experiments",
            get(routes::epigenome::experiments),
        )
        .route(
            "/v2/epigenome/experiment/{experiment_id}",
            get(routes::epigenome::experiment),
        )
        .route("/v2/epigenome/peaks", get(routes::epigenome::peaks))
        .route(
            "/v2/gene/id/{gene_id}/epigenome",
            get(routes::epigenome::gene_epigenome),
        );

    if let Some(root) = epigenome_signal_root.as_ref() {
        router = router.nest_service("/epigenome/signal", ServeDir::new(root));
    }

    router.layer(CorsLayer::permissive()).with_state(AppState {
        service,
        expression_repository,
        epigenome_repository,
        epigenome_base_path,
        default_assembly_accession,
        blast_jobs,
        blastp_jobs,
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
    /// Additional FASTA file to load into the refget reference (e.g. a protein
    /// FASTA whose records are addressable by their refget checksum).
    #[arg(long)]
    protein_fasta: Option<PathBuf>,
    #[arg(long)]
    no_fasta: bool,
    #[arg(long)]
    expression_snapshot: Option<PathBuf>,
    /// Path to `epigenome_snapshot.json` produced by `portal-cli import
    /// epigenome-manifest`. Without this flag, epigenome endpoints return
    /// empty results.
    #[arg(long)]
    epigenome_snapshot: Option<PathBuf>,
    /// Filesystem root holding bigWig signal files. Mounted at
    /// `/epigenome/signal` via tower-http's `ServeDir` (Range-aware).
    #[arg(long)]
    epigenome_signal_root: Option<PathBuf>,
    /// Optional override for the URL prefix the API emits when constructing
    /// signal URLs. Defaults to `/epigenome/signal`. Useful when bigWigs are
    /// served by a CDN.
    #[arg(long)]
    epigenome_base_path: Option<String>,
    #[arg(long)]
    blast_db_prefix: Option<PathBuf>,
    #[arg(long)]
    blastp_db_prefix: Option<PathBuf>,
    #[arg(long, default_value = "target/debug/worker")]
    blast_worker_bin: PathBuf,
    #[arg(long, default_value = "target/api/blast")]
    blast_work_dir: PathBuf,
    #[arg(long, default_value = "blastn")]
    blastn: PathBuf,
    #[arg(long, default_value = "blastp")]
    blastp: PathBuf,
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
        routes::jbrowse::default_config,
        routes::jbrowse::config,
        routes::jbrowse::chrom_sizes,
        routes::jbrowse::features,
        routes::genome::assembly,
        routes::genome::assembly_sequences,
        routes::genome::taxon,
        routes::gene::gene,
        routes::gene::gene_kegg,
        routes::gene::gene_orthogroups,
        routes::gene::orthogroup,
        routes::gene::kegg_pathways,
        routes::gene::kegg_pathway,
        routes::expression::gene_expression,
        routes::expression::clustergram,
        routes::epigenome::experiments,
        routes::epigenome::experiment,
        routes::epigenome::peaks,
        routes::epigenome::gene_epigenome,
        routes::enrichment::analysis,
        routes::gene::gene_search,
        routes::blast::create_blastn_job,
        routes::blast::blastn_job,
        routes::blast::create_blastp_job,
        routes::blast::blastp_job,
        protein::sequence,
        sequence::segments,
        routes::genome::region_features,
        refget::service_info,
        refget::sequence,
    ),
    components(schemas(
        ErrorResponse,
        routes::blast::AnnotatedHomologyHitResponse,
        routes::blast::AnnotatedHomologySearchResultResponse,
        routes::blast::BlastnJobRequest,
        routes::blast::BlastnJobResponse,
        routes::blast::BlastpJobRequest,
        routes::blast::JobStatusResponse,
        routes::expression::GeneExpressionPoint,
        routes::expression::GeneExpressionQuery,
        routes::expression::ExpressionClustergramQuery,
        routes::expression::ExpressionClustergramResponse,
        routes::expression::ExpressionGeneLabel,
        routes::expression::ExpressionSampleLabel,
        routes::epigenome::EpigenomeExperimentSummary,
        routes::epigenome::EpigenomeExperimentDetail,
        routes::epigenome::EpigenomePeakHit,
        routes::epigenome::EpigenomeExperimentWithPeaks,
        routes::epigenome::EpigenomeGeneView,
        routes::epigenome::PublicPeak,
        routes::epigenome::PublicRegion,
        routes::epigenome::ExperimentListQuery,
        routes::epigenome::EpigenomePeaksQuery,
        routes::epigenome::GeneEpigenomeQuery,
        epigenome_domain::Antibody,
        epigenome_domain::Assay,
        epigenome_domain::Experiment,
        epigenome_domain::ExperimentId,
        epigenome_domain::ExperimentQc,
        epigenome_domain::GeoSampleAccession,
        epigenome_domain::GeoSeriesAccession,
        epigenome_domain::Peak,
        epigenome_domain::PeakKind,
        epigenome_domain::Target,
        expression_domain::SraRunAccession,
        routes::enrichment::EnrichmentAnalysisRequest,
        routes::enrichment::EnrichmentAnalysisResponse,
        routes::enrichment::EnrichmentAnnotationKind,
        routes::enrichment::EnrichmentTerm,
        routes::enrichment::EnrichmentTermResult,
        routes::gene::GeneSearchQuery,
        routes::genome::TaxonResponse,
        routes::jbrowse::JBrowseAssembly,
        routes::jbrowse::JBrowseChromSizesAdapter,
        routes::jbrowse::JBrowseConfigQuery,
        routes::jbrowse::JBrowseDefaultSession,
        routes::jbrowse::JBrowseDefaultView,
        routes::jbrowse::JBrowseDefaultViewInit,
        routes::jbrowse::JBrowseFeature,
        routes::jbrowse::JBrowseFeaturesQuery,
        routes::jbrowse::JBrowsePortalConfig,
        routes::jbrowse::JBrowseRendering,
        routes::jbrowse::JBrowseRootConfig,
        routes::jbrowse::JBrowseSequenceTrack,
        routes::jbrowse::JBrowseUriLocation,
        protein::ProteinQuery,
        refget::RefgetQuery,
        refget::RefgetServiceInfo,
        sequence::SequenceOutputFormat,
        sequence::SequenceSegmentsQuery,
        HealthResponse,
        expression_domain::ExpressionUnit,
        genome_domain::AnnotationEvidence,
        genome_domain::AnnotationSource,
        genome_domain::Assembly,
        genome_domain::AssemblyAccession,
        genome_domain::AssemblySource,
        genome_domain::Cds,
        genome_domain::ClosedRegion,
        genome_domain::Exon,
        genome_domain::FunctionalAnnotation,
        genome_domain::Gene,
        genome_domain::GeneId,
        genome_domain::GeneRecord,
        genome_domain::GoNamespace,
        genome_domain::GoTermAnnotation,
        genome_domain::GoTermId,
        genome_domain::HalfOpenRegion,
        genome_domain::HomologyHit,
        genome_domain::HomologySearchMethod,
        genome_domain::InterProAnnotation,
        genome_domain::InterProId,
        genome_domain::KeggAnnotation,
        genome_domain::KeggCatalog,
        genome_domain::KeggEntryId,
        genome_domain::KeggEntryKind,
        genome_domain::KeggKoLinks,
        genome_domain::KeggModule,
        genome_domain::KeggModuleId,
        genome_domain::KeggPathway,
        genome_domain::KeggPathwayId,
        genome_domain::KeggReaction,
        genome_domain::KeggReactionId,
        GeneKeggOrthologyEntry,
        GeneKeggView,
        KeggGeneSummary,
        KeggPathwayDetail,
        KeggPathwayKoEntry,
        KeggPathwaySummary,
        genome_domain::KogAnnotation,
        genome_domain::KogEntryId,
        genome_domain::NcbiFamAccession,
        genome_domain::NcbiFamAnnotation,
        genome_domain::Orthogroup,
        genome_domain::OrthogroupCatalog,
        genome_domain::OrthogroupId,
        genome_domain::OrthogroupMember,
        genome_domain::PfamAccession,
        genome_domain::PfamAnnotation,
        genome_domain::Position0,
        genome_domain::Position1,
        genome_domain::Sequence,
        genome_domain::SequenceName,
        genome_domain::Strand,
        genome_domain::TaxId,
        genome_domain::Taxon,
        genome_domain::Transcript,
        genome_domain::TranscriptId,
        co_expression::ClusterDendrogram,
        co_expression::ClusterDendrogramNode,
    )),
    tags((name = "plant-genome-portal", description = "Plant Genome Portal API"))
)]
struct ApiDoc;

#[derive(Debug, Serialize, ToSchema)]
struct HealthResponse {
    ok: bool,
}

#[derive(Debug)]
pub(crate) enum ApiError {
    Service(ServiceError),
    Job(JobManagerError),
    BlastUnavailable(String),
}

impl From<ServiceError> for ApiError {
    fn from(error: ServiceError) -> Self {
        Self::Service(error)
    }
}

impl From<JobManagerError> for ApiError {
    fn from(error: JobManagerError) -> Self {
        Self::Job(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Service(
                ServiceError::TaxonNotFound(_)
                | ServiceError::AssemblyNotFound(_)
                | ServiceError::GeneNotFound(_)
                | ServiceError::TranscriptNotFound(_)
                | ServiceError::SequenceNotFound(_)
                | ServiceError::ProteinSequenceUnavailable(_)
                | ServiceError::KeggPathwayNotFound(_)
                | ServiceError::OrthogroupNotFound(_),
            )
            | Self::Job(JobManagerError::JobNotFound(_)) => StatusCode::NOT_FOUND,
            Self::Service(ServiceError::InvalidRequest(_)) => StatusCode::BAD_REQUEST,
            Self::Job(JobManagerError::SubmissionFailed(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BlastUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        let error = match self {
            Self::Service(error) => error.to_string(),
            Self::Job(error) => error.to_string(),
            Self::BlastUnavailable(error) => error,
        };

        (status, Json(ErrorResponse { error })).into_response()
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ErrorResponse {
    error: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use genome_domain::TaxId;

    fn status_of(error: ApiError) -> StatusCode {
        error.into_response().status()
    }

    #[test]
    fn service_not_found_variants_map_to_404() {
        // Every "not found" variant must surface as 404. The match in
        // `into_response` is exhaustive (no wildcard), so adding a new
        // ServiceError variant forces a compile error there — this test pins the
        // status for each variant that currently exists.
        let not_found = [
            ServiceError::TaxonNotFound(TaxId::new(1)),
            ServiceError::AssemblyNotFound("a".to_owned()),
            ServiceError::GeneNotFound("g".to_owned()),
            ServiceError::TranscriptNotFound("t".to_owned()),
            ServiceError::SequenceNotFound("s".to_owned()),
            ServiceError::ProteinSequenceUnavailable("p".to_owned()),
            ServiceError::KeggPathwayNotFound("k".to_owned()),
            ServiceError::OrthogroupNotFound("o".to_owned()),
        ];

        for error in not_found {
            assert_eq!(status_of(ApiError::Service(error)), StatusCode::NOT_FOUND);
        }
    }

    #[test]
    fn invalid_request_maps_to_400() {
        assert_eq!(
            status_of(ApiError::Service(ServiceError::InvalidRequest(
                "bad".to_owned()
            ))),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn job_errors_map_to_404_and_500() {
        assert_eq!(
            status_of(ApiError::Job(JobManagerError::JobNotFound("j".to_owned()))),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_of(ApiError::Job(JobManagerError::SubmissionFailed(
                "boom".to_owned()
            ))),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn blast_unavailable_maps_to_503() {
        assert_eq!(
            status_of(ApiError::BlastUnavailable("no blast".to_owned())),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[tokio::test]
    async fn error_response_body_carries_the_display_message() {
        let response =
            ApiError::Service(ServiceError::GeneNotFound("Mp9".to_owned())).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(parsed["error"], "gene not found: Mp9");
    }
}
