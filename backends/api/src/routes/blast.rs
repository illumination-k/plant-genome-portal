use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use genome_core::AssemblyAccession;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use service::{
    AnnotatedHomologySearchResult, InMemoryJobManager, JobExecutor, JobManager, JobRecord,
    JobStatus, ServiceError, WorkerJob,
};
use std::fs;
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Stdio};
use utoipa::ToSchema;

use crate::{ApiError, AppState};

pub(crate) type BlastJobManager = InMemoryJobManager<BlastJobInput, AnnotatedHomologySearchResult>;
pub(crate) type BlastpJobManager = InMemoryJobManager<BlastJobInput, AnnotatedHomologySearchResult>;

#[utoipa::path(
    post,
    path = "/v2/tools/blastn/jobs",
    request_body = BlastnJobRequest,
    responses(
        (status = 202, description = "BLASTN job accepted", body = BlastnJobResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
        (status = 503, description = "BLASTN worker is not configured", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn create_blastn_job(
    State(state): State<AppState>,
    Json(request): Json<BlastnJobRequest>,
) -> Result<(StatusCode, Json<BlastnJobResponse>), ApiError> {
    submit_blast_job(
        state.blast_jobs.as_ref(),
        "BLASTN worker is not configured",
        "homology.blastn",
        request.into_job_input()?,
    )
}

#[utoipa::path(
    get,
    path = "/v2/tools/blastn/jobs/{job_id}",
    params(("job_id" = String, Path, description = "BLASTN job identifier")),
    responses(
        (status = 200, description = "BLASTN job status", body = BlastnJobResponse),
        (status = 404, description = "Job not found", body = crate::ErrorResponse),
        (status = 503, description = "BLASTN worker is not configured", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn blastn_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<BlastnJobResponse>, ApiError> {
    get_blast_job(
        state.blast_jobs.as_ref(),
        "BLASTN worker is not configured",
        &job_id,
    )
}

#[utoipa::path(
    post,
    path = "/v2/tools/blastp/jobs",
    request_body = BlastpJobRequest,
    responses(
        (status = 202, description = "BLASTP job accepted", body = BlastnJobResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
        (status = 503, description = "BLASTP worker is not configured", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn create_blastp_job(
    State(state): State<AppState>,
    Json(request): Json<BlastpJobRequest>,
) -> Result<(StatusCode, Json<BlastnJobResponse>), ApiError> {
    submit_blast_job(
        state.blastp_jobs.as_ref(),
        "BLASTP worker is not configured",
        "homology.blastp",
        request.into_job_input()?,
    )
}

#[utoipa::path(
    get,
    path = "/v2/tools/blastp/jobs/{job_id}",
    params(("job_id" = String, Path, description = "BLASTP job identifier")),
    responses(
        (status = 200, description = "BLASTP job status", body = BlastnJobResponse),
        (status = 404, description = "Job not found", body = crate::ErrorResponse),
        (status = 503, description = "BLASTP worker is not configured", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn blastp_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<BlastnJobResponse>, ApiError> {
    get_blast_job(
        state.blastp_jobs.as_ref(),
        "BLASTP worker is not configured",
        &job_id,
    )
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlastnJobRequest {
    assembly_accession: String,
    query: String,
    task: Option<String>,
    evalue: Option<f64>,
    max_target_seqs: Option<usize>,
}

impl BlastnJobRequest {
    fn into_job_input(self) -> Result<BlastJobInput, ApiError> {
        let request = ValidatedBlastJobRequest::new(self.into(), "blastn", validate_blastn_task)?;
        Ok(BlastJobInput::from(request))
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlastpJobRequest {
    assembly_accession: String,
    query: String,
    task: Option<String>,
    evalue: Option<f64>,
    max_target_seqs: Option<usize>,
}

impl BlastpJobRequest {
    fn into_job_input(self) -> Result<BlastJobInput, ApiError> {
        let request = ValidatedBlastJobRequest::new(self.into(), "blastp", validate_blastp_task)?;
        Ok(BlastJobInput::from(request))
    }
}

struct RawBlastJobRequest {
    assembly_accession: String,
    query: String,
    task: Option<String>,
    evalue: Option<f64>,
    max_target_seqs: Option<usize>,
}

impl RawBlastJobRequest {
    fn new(
        assembly_accession: String,
        query: String,
        task: Option<String>,
        evalue: Option<f64>,
        max_target_seqs: Option<usize>,
    ) -> Self {
        Self {
            assembly_accession,
            query,
            task,
            evalue,
            max_target_seqs,
        }
    }
}

impl From<BlastnJobRequest> for RawBlastJobRequest {
    fn from(request: BlastnJobRequest) -> Self {
        Self::new(
            request.assembly_accession,
            request.query,
            request.task,
            request.evalue,
            request.max_target_seqs,
        )
    }
}

impl From<BlastpJobRequest> for RawBlastJobRequest {
    fn from(request: BlastpJobRequest) -> Self {
        Self::new(
            request.assembly_accession,
            request.query,
            request.task,
            request.evalue,
            request.max_target_seqs,
        )
    }
}

struct ValidatedBlastJobRequest {
    assembly_accession: AssemblyAccession,
    query: String,
    task: String,
    evalue: f64,
    max_target_seqs: usize,
}

impl ValidatedBlastJobRequest {
    fn new(
        raw: RawBlastJobRequest,
        default_task: &str,
        validate_task: fn(&str) -> Result<(), ApiError>,
    ) -> Result<Self, ApiError> {
        let assembly_accession = AssemblyAccession::new(&raw.assembly_accession)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
        let task = raw.task.unwrap_or_else(|| default_task.to_owned());
        validate_task(&task)?;
        let evalue = valid_evalue(raw.evalue)?;
        let max_target_seqs = valid_max_target_seqs(raw.max_target_seqs)?;
        if raw.query.trim().is_empty() {
            return Err(ServiceError::InvalidRequest("query must not be empty".to_owned()).into());
        }

        Ok(Self {
            assembly_accession,
            query: raw.query,
            task,
            evalue,
            max_target_seqs,
        })
    }
}

fn valid_evalue(value: Option<f64>) -> Result<f64, ApiError> {
    let evalue = value.unwrap_or(10.0);
    if evalue <= 0.0 {
        return Err(
            ServiceError::InvalidRequest("evalue must be greater than zero".to_owned()).into(),
        );
    }
    Ok(evalue)
}

fn valid_max_target_seqs(value: Option<usize>) -> Result<usize, ApiError> {
    let max_target_seqs = value.unwrap_or(50);
    if max_target_seqs == 0 {
        return Err(ServiceError::InvalidRequest(
            "maxTargetSeqs must be greater than zero".to_owned(),
        )
        .into());
    }
    Ok(max_target_seqs)
}

fn submit_blast_job<I>(
    manager: Option<&InMemoryJobManager<I, AnnotatedHomologySearchResult>>,
    unavailable_message: &str,
    kind: &str,
    input: I,
) -> Result<(StatusCode, Json<BlastnJobResponse>), ApiError>
where
    I: Send + 'static,
{
    let manager =
        manager.ok_or_else(|| ApiError::BlastUnavailable(unavailable_message.to_owned()))?;
    let record = manager.submit(kind.to_owned(), input)?;
    Ok((StatusCode::ACCEPTED, Json(record.into())))
}

fn get_blast_job<I>(
    manager: Option<&InMemoryJobManager<I, AnnotatedHomologySearchResult>>,
    unavailable_message: &str,
    job_id: &str,
) -> Result<Json<BlastnJobResponse>, ApiError>
where
    I: Send + 'static,
{
    let manager =
        manager.ok_or_else(|| ApiError::BlastUnavailable(unavailable_message.to_owned()))?;
    Ok(Json(manager.get(job_id)?.into()))
}

fn validate_blastn_task(task: &str) -> Result<(), ApiError> {
    if matches!(
        task,
        "blastn" | "blastn-short" | "megablast" | "dc-megablast"
    ) {
        Ok(())
    } else {
        Err(ServiceError::InvalidRequest(format!("unsupported BLASTN task: {task}")).into())
    }
}

fn validate_blastp_task(task: &str) -> Result<(), ApiError> {
    if matches!(task, "blastp" | "blastp-short" | "blastp-fast") {
        Ok(())
    } else {
        Err(ServiceError::InvalidRequest(format!("unsupported BLASTP task: {task}")).into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlastJobInput {
    assembly_accession: AssemblyAccession,
    query: String,
    task: String,
    evalue: f64,
    max_target_seqs: usize,
}

impl From<ValidatedBlastJobRequest> for BlastJobInput {
    fn from(request: ValidatedBlastJobRequest) -> Self {
        Self {
            assembly_accession: request.assembly_accession,
            query: request.query,
            task: request.task,
            evalue: request.evalue,
            max_target_seqs: request.max_target_seqs,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum BlastMethod {
    Blastn,
    Blastp,
}

impl BlastMethod {
    fn subcommand(self) -> &'static str {
        match self {
            Self::Blastn => "blastn-job",
            Self::Blastp => "blastp-job",
        }
    }

    fn program_flag(self) -> &'static str {
        match self {
            Self::Blastn => "--blastn",
            Self::Blastp => "--blastp",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BlastWorkerCommand {
    pub(crate) worker_bin: PathBuf,
    pub(crate) blast_db_prefix: PathBuf,
    pub(crate) work_dir: PathBuf,
    pub(crate) program: PathBuf,
    pub(crate) snapshot: Option<PathBuf>,
    pub(crate) method: BlastMethod,
}

impl BlastWorkerCommand {
    fn dispatch(
        &self,
        job_id: String,
        kind: String,
        worker_input: BlastWorkerInput,
    ) -> Result<AnnotatedHomologySearchResult, String> {
        fs::create_dir_all(&self.work_dir).map_err(|error| error.to_string())?;
        let input_path = self.work_dir.join(format!("{job_id}.input.msgpack"));
        let output_path = self.work_dir.join(format!("{job_id}.output.msgpack"));
        let worker_job = WorkerJob {
            id: job_id,
            kind,
            payload: worker_input,
        };

        fs::write(&input_path, encode_message_pack(&worker_job)?)
            .map_err(|error| error.to_string())?;

        let output = ProcessCommand::new(&self.worker_bin)
            .arg(self.method.subcommand())
            .arg("--blast-db-prefix")
            .arg(&self.blast_db_prefix)
            .arg("--work-dir")
            .arg(&self.work_dir)
            .arg(self.method.program_flag())
            .arg(&self.program)
            .arg("--input")
            .arg(&input_path)
            .arg("--output")
            .arg(&output_path)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|error| format!("failed to start worker: {error}"))?;

        let _ = fs::remove_file(&input_path);

        if !output.status.success() {
            let _ = fs::remove_file(&output_path);
            return Err(format!(
                "worker exited with status {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let result_bytes = fs::read(&output_path).map_err(|error| error.to_string())?;
        let _ = fs::remove_file(&output_path);
        decode_message_pack(&result_bytes)
    }
}

impl JobExecutor<BlastJobInput, AnnotatedHomologySearchResult> for BlastWorkerCommand {
    fn execute(
        &self,
        job: WorkerJob<BlastJobInput>,
    ) -> Result<AnnotatedHomologySearchResult, String> {
        self.dispatch(
            job.id,
            job.kind,
            BlastWorkerInput {
                assembly_accession: job.payload.assembly_accession,
                query: job.payload.query,
                task: job.payload.task,
                evalue: job.payload.evalue,
                max_target_seqs: job.payload.max_target_seqs,
                snapshot: self.snapshot.clone(),
            },
        )
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BlastWorkerInput {
    assembly_accession: AssemblyAccession,
    query: String,
    task: String,
    evalue: f64,
    max_target_seqs: usize,
    snapshot: Option<PathBuf>,
}

fn encode_message_pack<T>(value: &T) -> Result<Vec<u8>, String>
where
    T: Serialize,
{
    rmp_serde::to_vec_named(value).map_err(|error| error.to_string())
}

fn decode_message_pack<T>(bytes: &[u8]) -> Result<T, String>
where
    T: DeserializeOwned,
{
    rmp_serde::from_slice(bytes).map_err(|error| error.to_string())
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlastnJobResponse {
    id: String,
    kind: String,
    status: JobStatusResponse,
    result: Option<AnnotatedHomologySearchResultResponse>,
    error: Option<String>,
}

impl From<JobRecord<AnnotatedHomologySearchResult>> for BlastnJobResponse {
    fn from(record: JobRecord<AnnotatedHomologySearchResult>) -> Self {
        Self {
            id: record.id,
            kind: record.kind,
            status: record.status.into(),
            result: record.output.map(Into::into),
            error: record.error,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum JobStatusResponse {
    Queued,
    Running,
    Succeeded,
    Failed,
}

impl From<JobStatus> for JobStatusResponse {
    fn from(status: JobStatus) -> Self {
        match status {
            JobStatus::Queued => Self::Queued,
            JobStatus::Running => Self::Running,
            JobStatus::Succeeded => Self::Succeeded,
            JobStatus::Failed => Self::Failed,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnnotatedHomologySearchResultResponse {
    method: genome_core::HomologySearchMethod,
    task: String,
    hits: Vec<AnnotatedHomologyHitResponse>,
}

impl From<AnnotatedHomologySearchResult> for AnnotatedHomologySearchResultResponse {
    fn from(result: AnnotatedHomologySearchResult) -> Self {
        Self {
            method: result.method,
            task: result.task,
            hits: result.hits.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnnotatedHomologyHitResponse {
    hit: genome_core::HomologyHit,
    overlapping_gene_ids: Vec<genome_core::GeneId>,
}

impl From<service::AnnotatedHomologyHit> for AnnotatedHomologyHitResponse {
    fn from(hit: service::AnnotatedHomologyHit) -> Self {
        Self {
            hit: hit.hit,
            overlapping_gene_ids: hit.overlapping_gene_ids,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn validate_blastp_task_accepts_supported_tasks_and_rejects_others() {
        assert!(validate_blastp_task("blastp").is_ok());
        assert!(validate_blastp_task("blastp-short").is_ok());
        assert!(validate_blastp_task("blastp-fast").is_ok());
        assert!(matches!(
            validate_blastp_task("blastn"),
            Err(ApiError::Service(ServiceError::InvalidRequest(_)))
        ));
        assert!(matches!(
            validate_blastp_task(""),
            Err(ApiError::Service(ServiceError::InvalidRequest(_)))
        ));
    }

    fn valid_blastp_request() -> BlastpJobRequest {
        BlastpJobRequest {
            assembly_accession: "GCA_test".to_owned(),
            query: "MVTAG".to_owned(),
            task: None,
            evalue: None,
            max_target_seqs: None,
        }
    }

    #[test]
    fn blastp_request_defaults_task_to_blastp_and_uses_default_thresholds() {
        let input = valid_blastp_request().into_job_input().unwrap();
        assert_eq!(input.task, "blastp");
        assert_eq!(input.evalue, 10.0);
        assert_eq!(input.max_target_seqs, 50);
        assert_eq!(input.assembly_accession.as_str(), "GCA_test");
    }

    #[test]
    fn blastp_request_rejects_non_positive_evalue() {
        let mut request = valid_blastp_request();
        request.evalue = Some(0.0);
        assert!(matches!(
            request.into_job_input(),
            Err(ApiError::Service(ServiceError::InvalidRequest(_)))
        ));
        let mut request = valid_blastp_request();
        request.evalue = Some(-1.0);
        assert!(matches!(
            request.into_job_input(),
            Err(ApiError::Service(ServiceError::InvalidRequest(_)))
        ));
    }

    #[test]
    fn blastp_request_rejects_zero_max_target_seqs() {
        let mut request = valid_blastp_request();
        request.max_target_seqs = Some(0);
        assert!(matches!(
            request.into_job_input(),
            Err(ApiError::Service(ServiceError::InvalidRequest(_)))
        ));
    }

    #[test]
    fn blastp_request_rejects_empty_query() {
        let mut request = valid_blastp_request();
        request.query = "   ".to_owned();
        assert!(matches!(
            request.into_job_input(),
            Err(ApiError::Service(ServiceError::InvalidRequest(_)))
        ));
    }

    #[test]
    fn blastp_request_rejects_unsupported_task() {
        let mut request = valid_blastp_request();
        request.task = Some("blastn".to_owned());
        assert!(matches!(
            request.into_job_input(),
            Err(ApiError::Service(ServiceError::InvalidRequest(_)))
        ));
    }

    #[test]
    fn blast_method_subcommand_matches_worker_cli_names() {
        assert_eq!(BlastMethod::Blastn.subcommand(), "blastn-job");
        assert_eq!(BlastMethod::Blastp.subcommand(), "blastp-job");
    }

    #[test]
    fn blast_method_program_flag_matches_worker_cli_flags() {
        assert_eq!(BlastMethod::Blastn.program_flag(), "--blastn");
        assert_eq!(BlastMethod::Blastp.program_flag(), "--blastp");
    }
}
