use genome_core::{
    AssemblyAccession, HomologyHit, HomologySearchMethod, Position1, SequenceName, TranscriptId,
};
use serde::{Deserialize, Serialize};
use service::{
    AnnotatedHomologyHit, AnnotatedHomologySearchResult, HomologyService, Worker, WorkerJob,
};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use storage::FileGenomeRepository;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct BlastRunner {
    program: PathBuf,
    db_prefix: PathBuf,
    work_dir: PathBuf,
    method: HomologySearchMethod,
}

impl BlastRunner {
    pub fn blastn(
        db_prefix: PathBuf,
        work_dir: PathBuf,
        program: PathBuf,
    ) -> Result<Self, BlastWorkerError> {
        Self::new(db_prefix, work_dir, program, HomologySearchMethod::Blastn)
    }

    pub fn blastp(
        db_prefix: PathBuf,
        work_dir: PathBuf,
        program: PathBuf,
    ) -> Result<Self, BlastWorkerError> {
        Self::new(db_prefix, work_dir, program, HomologySearchMethod::Blastp)
    }

    fn new(
        db_prefix: PathBuf,
        work_dir: PathBuf,
        program: PathBuf,
        method: HomologySearchMethod,
    ) -> Result<Self, BlastWorkerError> {
        fs::create_dir_all(&work_dir)?;
        Ok(Self {
            program,
            db_prefix,
            work_dir,
            method,
        })
    }

    pub fn search(
        &self,
        input: BlastHomologySearchInput,
    ) -> Result<AnnotatedHomologySearchResult, BlastWorkerError> {
        input.validate(&self.method)?;
        let query_path = self.write_query_file(&input.query)?;
        let result = self.run_blast(&query_path, &input);
        let _ = fs::remove_file(&query_path);
        result
    }

    fn write_query_file(&self, query: &str) -> Result<PathBuf, BlastWorkerError> {
        let normalized = normalize_query(query, &self.method)?;
        let path = self.work_dir.join(format!("query-{}.fa", unique_suffix()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(normalized.as_bytes())?;
        Ok(path)
    }

    fn run_blast(
        &self,
        query_path: &Path,
        input: &BlastHomologySearchInput,
    ) -> Result<AnnotatedHomologySearchResult, BlastWorkerError> {
        let output = Command::new(&self.program)
            .arg("-query")
            .arg(query_path)
            .arg("-db")
            .arg(&self.db_prefix)
            .arg("-task")
            .arg(&input.task)
            .arg("-evalue")
            .arg(input.evalue.to_string())
            .arg("-max_target_seqs")
            .arg(input.max_target_seqs.clamp(1, 500).to_string())
            .arg("-outfmt")
            .arg("6 qseqid sseqid pident length mismatch gapopen qstart qend sstart send evalue bitscore qseq sseq")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|source| BlastWorkerError::CommandStart {
                program: self.program.clone(),
                source,
            })?;

        if !output.status.success() {
            return Err(BlastWorkerError::CommandFailed {
                program: self.program.clone(),
                status: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        let stdout = String::from_utf8(output.stdout)?;
        let hits = parse_tabular_hits(&stdout, &input.assembly_accession, &self.method)?;
        let result = genome_core::HomologySearchResult {
            method: self.method.clone(),
            task: input.task.clone(),
            hits,
        };

        if let Some(snapshot) = &input.snapshot {
            let repository = FileGenomeRepository::from_snapshot_path(snapshot)?;
            return Ok(HomologyService::new(repository).annotate_result(result));
        }

        Ok(AnnotatedHomologySearchResult {
            method: result.method,
            task: result.task,
            hits: result
                .hits
                .into_iter()
                .map(|hit| AnnotatedHomologyHit {
                    hit,
                    overlapping_gene_ids: Vec::new(),
                })
                .collect(),
        })
    }
}

impl Worker for BlastRunner {
    type Input = BlastHomologySearchInput;
    type Output = AnnotatedHomologySearchResult;
    type Error = BlastWorkerError;

    fn run(&self, job: WorkerJob<Self::Input>) -> Result<Self::Output, Self::Error> {
        self.search(job.payload)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlastHomologySearchInput {
    pub assembly_accession: AssemblyAccession,
    pub query: String,
    pub task: String,
    pub evalue: f64,
    pub max_target_seqs: usize,
    pub snapshot: Option<PathBuf>,
}

impl BlastHomologySearchInput {
    fn validate(&self, method: &HomologySearchMethod) -> Result<(), BlastWorkerError> {
        normalize_query(&self.query, method)?;
        validate_task(&self.task, method)?;
        if self.evalue <= 0.0 {
            return Err(BlastWorkerError::InvalidRequest(
                "evalue must be greater than zero".to_owned(),
            ));
        }
        if self.max_target_seqs == 0 {
            return Err(BlastWorkerError::InvalidRequest(
                "max_target_seqs must be greater than zero".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum BlastWorkerError {
    #[error("invalid BLAST request: {0}")]
    InvalidRequest(String),
    #[error("failed to start {program}: {source}")]
    CommandStart { program: PathBuf, source: io::Error },
    #[error("{program} exited with status {status:?}: {stderr}")]
    CommandFailed {
        program: PathBuf,
        status: Option<i32>,
        stderr: String,
    },
    #[error("invalid BLAST output: {0}")]
    InvalidOutput(String),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Storage(#[from] storage::StorageError),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}

fn normalize_query(query: &str, method: &HomologySearchMethod) -> Result<String, BlastWorkerError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(BlastWorkerError::InvalidRequest(
            "query sequence must not be empty".to_owned(),
        ));
    }

    if query.starts_with('>') {
        return Ok(format!("{query}\n"));
    }

    let kind_label = match method {
        HomologySearchMethod::Blastn => "nucleotide",
        HomologySearchMethod::Blastp => "amino acid",
    };
    let is_valid = match method {
        HomologySearchMethod::Blastn => is_iupac_nucleotide,
        HomologySearchMethod::Blastp => is_iupac_amino_acid,
    };

    let mut sequence = String::with_capacity(query.len());
    for ch in query.chars().filter(|ch| !ch.is_whitespace()) {
        if !is_valid(ch) {
            return Err(BlastWorkerError::InvalidRequest(format!(
                "query contains a non-{kind_label} character: {ch}"
            )));
        }
        sequence.push(ch.to_ascii_uppercase());
    }

    if sequence.is_empty() {
        return Err(BlastWorkerError::InvalidRequest(format!(
            "query sequence must contain {kind_label}s"
        )));
    }

    Ok(format!(">query\n{sequence}\n"))
}

fn is_iupac_nucleotide(ch: char) -> bool {
    matches!(
        ch.to_ascii_uppercase(),
        'A' | 'C'
            | 'G'
            | 'T'
            | 'U'
            | 'R'
            | 'Y'
            | 'S'
            | 'W'
            | 'K'
            | 'M'
            | 'B'
            | 'D'
            | 'H'
            | 'V'
            | 'N'
    )
}

fn is_iupac_amino_acid(ch: char) -> bool {
    matches!(
        ch.to_ascii_uppercase(),
        'A' | 'B'
            | 'C'
            | 'D'
            | 'E'
            | 'F'
            | 'G'
            | 'H'
            | 'I'
            | 'J'
            | 'K'
            | 'L'
            | 'M'
            | 'N'
            | 'O'
            | 'P'
            | 'Q'
            | 'R'
            | 'S'
            | 'T'
            | 'U'
            | 'V'
            | 'W'
            | 'Y'
            | 'X'
            | 'Z'
            | '*'
    )
}

fn validate_task(task: &str, method: &HomologySearchMethod) -> Result<(), BlastWorkerError> {
    let supported = match method {
        HomologySearchMethod::Blastn => matches!(
            task,
            "blastn" | "blastn-short" | "megablast" | "dc-megablast"
        ),
        HomologySearchMethod::Blastp => {
            matches!(task, "blastp" | "blastp-short" | "blastp-fast")
        }
    };
    if supported {
        Ok(())
    } else {
        let label = match method {
            HomologySearchMethod::Blastn => "BLASTN",
            HomologySearchMethod::Blastp => "BLASTP",
        };
        Err(BlastWorkerError::InvalidRequest(format!(
            "unsupported {label} task: {task}"
        )))
    }
}

fn parse_tabular_hits(
    output: &str,
    assembly_accession: &AssemblyAccession,
    method: &HomologySearchMethod,
) -> Result<Vec<HomologyHit>, BlastWorkerError> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse_tabular_hit(line, assembly_accession, method))
        .collect()
}

fn parse_tabular_hit(
    line: &str,
    assembly_accession: &AssemblyAccession,
    method: &HomologySearchMethod,
) -> Result<HomologyHit, BlastWorkerError> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != 14 {
        return Err(BlastWorkerError::InvalidOutput(format!(
            "expected 14 fields, got {}",
            fields.len()
        )));
    }

    let percent_identity = parse_f64(fields[2], "pident")?;
    let alignment_length = parse_u64(fields[3], "length")?;
    let mismatches = parse_u64(fields[4], "mismatch")?;
    let gap_opens = parse_u64(fields[5], "gapopen")?;
    let query_start =
        Position1::new(parse_u64(fields[6], "qstart")?).map_err(invalid_domain_output)?;
    let query_end = Position1::new(parse_u64(fields[7], "qend")?).map_err(invalid_domain_output)?;
    let subject_start =
        Position1::new(parse_u64(fields[8], "sstart")?).map_err(invalid_domain_output)?;
    let subject_end =
        Position1::new(parse_u64(fields[9], "send")?).map_err(invalid_domain_output)?;
    let evalue = parse_f64(fields[10], "evalue")?;
    let bit_score = parse_f64(fields[11], "bitscore")?;

    match method {
        HomologySearchMethod::Blastn => HomologyHit::from_blastn_alignment(
            assembly_accession.clone(),
            fields[0].to_owned(),
            SequenceName::new(fields[1]).map_err(invalid_domain_output)?,
            percent_identity,
            alignment_length,
            mismatches,
            gap_opens,
            query_start,
            query_end,
            subject_start,
            subject_end,
            evalue,
            bit_score,
            fields[12].to_owned(),
            fields[13].to_owned(),
        )
        .map_err(invalid_domain_output),
        HomologySearchMethod::Blastp => HomologyHit::from_blastp_alignment(
            assembly_accession.clone(),
            fields[0].to_owned(),
            TranscriptId::new(fields[1]).map_err(invalid_domain_output)?,
            percent_identity,
            alignment_length,
            mismatches,
            gap_opens,
            query_start,
            query_end,
            subject_start,
            subject_end,
            evalue,
            bit_score,
            fields[12].to_owned(),
            fields[13].to_owned(),
        )
        .map_err(invalid_domain_output),
    }
}

fn parse_u64(value: &str, field: &str) -> Result<u64, BlastWorkerError> {
    value
        .parse()
        .map_err(|_| BlastWorkerError::InvalidOutput(format!("invalid {field}: {value}")))
}

fn parse_f64(value: &str, field: &str) -> Result<f64, BlastWorkerError> {
    value
        .parse()
        .map_err(|_| BlastWorkerError::InvalidOutput(format!("invalid {field}: {value}")))
}

fn invalid_domain_output(error: impl std::fmt::Display) -> BlastWorkerError {
    BlastWorkerError::InvalidOutput(error.to_string())
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{}-{nanos}", std::process::id())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn normalize_query_wraps_raw_nucleotide_sequence_as_fasta() {
        assert_eq!(
            normalize_query(" acgt\nnn ", &HomologySearchMethod::Blastn).unwrap(),
            ">query\nACGTNN\n"
        );
    }

    #[test]
    fn normalize_query_accepts_amino_acid_chars_only_for_blastp() {
        assert_eq!(
            normalize_query("MVTAG", &HomologySearchMethod::Blastp).unwrap(),
            ">query\nMVTAG\n"
        );
        assert!(normalize_query("MVTAG-stop", &HomologySearchMethod::Blastp).is_err());
    }

    #[test]
    fn normalize_query_rejects_amino_acids_under_blastn() {
        // 'P' is an amino acid letter that is not a nucleotide IUPAC code.
        assert!(normalize_query("ACGPT", &HomologySearchMethod::Blastn).is_err());
    }

    #[test]
    fn parse_tabular_hit_uses_domain_region_and_strand_for_blastn() {
        let hit = parse_tabular_hit(
            "query\tchr1\t99.5\t100\t1\t0\t1\t100\t500\t401\t1e-20\t80.0\tACGT\tACGT",
            &AssemblyAccession::new("GCA_test").unwrap(),
            &HomologySearchMethod::Blastn,
        )
        .unwrap();

        assert_eq!(hit.subject_region.start.get(), 401);
        assert_eq!(hit.subject_region.end.get(), 500);
        assert_eq!(hit.sequence_name.as_str(), "chr1");
        assert_eq!(hit.evalue, 1e-20);
    }

    #[test]
    fn parse_tabular_hit_treats_subject_as_transcript_id_for_blastp() {
        let hit = parse_tabular_hit(
            "query\tMp1g00010.1\t99.5\t100\t1\t0\t1\t100\t1\t100\t1e-50\t200.0\tMVTAG\tMVTAG",
            &AssemblyAccession::new("GCA_test").unwrap(),
            &HomologySearchMethod::Blastp,
        )
        .unwrap();

        assert_eq!(hit.sequence_name.as_str(), "Mp1g00010.1");
        assert_eq!(hit.subject_region.start.get(), 1);
        assert_eq!(hit.subject_region.end.get(), 100);
    }

    #[test]
    fn validate_task_distinguishes_blastn_and_blastp_tasks() {
        assert!(validate_task("blastn", &HomologySearchMethod::Blastn).is_ok());
        assert!(validate_task("blastp", &HomologySearchMethod::Blastn).is_err());
        assert!(validate_task("blastp", &HomologySearchMethod::Blastp).is_ok());
        assert!(validate_task("blastn", &HomologySearchMethod::Blastp).is_err());
        assert!(validate_task("blastn -remote", &HomologySearchMethod::Blastn).is_err());
    }
}
