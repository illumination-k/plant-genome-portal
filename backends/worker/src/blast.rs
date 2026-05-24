use genome_core::{AssemblyAccession, HomologyHit, HomologySearchMethod, Position1, SequenceName};
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
    blastn: PathBuf,
    db_prefix: PathBuf,
    work_dir: PathBuf,
}

impl BlastRunner {
    pub fn from_prepared(
        db_prefix: PathBuf,
        work_dir: PathBuf,
        blastn: PathBuf,
    ) -> Result<Self, BlastWorkerError> {
        fs::create_dir_all(&work_dir)?;
        Ok(Self {
            blastn,
            db_prefix,
            work_dir,
        })
    }

    pub fn search(
        &self,
        input: BlastHomologySearchInput,
    ) -> Result<AnnotatedHomologySearchResult, BlastWorkerError> {
        input.validate()?;
        let query_path = self.write_query_file(&input.query)?;
        let result = self.run_blastn(&query_path, &input);
        let _ = fs::remove_file(&query_path);
        result
    }

    fn write_query_file(&self, query: &str) -> Result<PathBuf, BlastWorkerError> {
        let normalized = normalize_query(query)?;
        let path = self.work_dir.join(format!("query-{}.fa", unique_suffix()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(normalized.as_bytes())?;
        Ok(path)
    }

    fn run_blastn(
        &self,
        query_path: &Path,
        input: &BlastHomologySearchInput,
    ) -> Result<AnnotatedHomologySearchResult, BlastWorkerError> {
        let output = Command::new(&self.blastn)
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
                program: self.blastn.clone(),
                source,
            })?;

        if !output.status.success() {
            return Err(BlastWorkerError::CommandFailed {
                program: self.blastn.clone(),
                status: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        let stdout = String::from_utf8(output.stdout)?;
        let hits = parse_tabular_hits(&stdout, &input.assembly_accession)?;
        let result = genome_core::HomologySearchResult {
            method: HomologySearchMethod::Blastn,
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
    fn validate(&self) -> Result<(), BlastWorkerError> {
        normalize_query(&self.query)?;
        validate_task(&self.task)?;
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

fn normalize_query(query: &str) -> Result<String, BlastWorkerError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(BlastWorkerError::InvalidRequest(
            "query sequence must not be empty".to_owned(),
        ));
    }

    if query.starts_with('>') {
        return Ok(format!("{query}\n"));
    }

    let mut sequence = String::with_capacity(query.len());
    for ch in query.chars().filter(|ch| !ch.is_whitespace()) {
        if !is_iupac_nucleotide(ch) {
            return Err(BlastWorkerError::InvalidRequest(format!(
                "query contains a non-nucleotide character: {ch}"
            )));
        }
        sequence.push(ch.to_ascii_uppercase());
    }

    if sequence.is_empty() {
        return Err(BlastWorkerError::InvalidRequest(
            "query sequence must contain nucleotides".to_owned(),
        ));
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

fn validate_task(task: &str) -> Result<(), BlastWorkerError> {
    if matches!(
        task,
        "blastn" | "blastn-short" | "megablast" | "dc-megablast"
    ) {
        Ok(())
    } else {
        Err(BlastWorkerError::InvalidRequest(format!(
            "unsupported BLASTN task: {task}"
        )))
    }
}

fn parse_tabular_hits(
    output: &str,
    assembly_accession: &AssemblyAccession,
) -> Result<Vec<HomologyHit>, BlastWorkerError> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse_tabular_hit(line, assembly_accession))
        .collect()
}

fn parse_tabular_hit(
    line: &str,
    assembly_accession: &AssemblyAccession,
) -> Result<HomologyHit, BlastWorkerError> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != 14 {
        return Err(BlastWorkerError::InvalidOutput(format!(
            "expected 14 fields, got {}",
            fields.len()
        )));
    }

    HomologyHit::from_blastn_alignment(
        assembly_accession.clone(),
        fields[0].to_owned(),
        SequenceName::new(fields[1]).map_err(invalid_domain_output)?,
        parse_f64(fields[2], "pident")?,
        parse_u64(fields[3], "length")?,
        parse_u64(fields[4], "mismatch")?,
        parse_u64(fields[5], "gapopen")?,
        Position1::new(parse_u64(fields[6], "qstart")?).map_err(invalid_domain_output)?,
        Position1::new(parse_u64(fields[7], "qend")?).map_err(invalid_domain_output)?,
        Position1::new(parse_u64(fields[8], "sstart")?).map_err(invalid_domain_output)?,
        Position1::new(parse_u64(fields[9], "send")?).map_err(invalid_domain_output)?,
        parse_f64(fields[10], "evalue")?,
        parse_f64(fields[11], "bitscore")?,
        fields[12].to_owned(),
        fields[13].to_owned(),
    )
    .map_err(invalid_domain_output)
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
    fn normalize_query_wraps_raw_sequence_as_fasta() {
        assert_eq!(normalize_query(" acgt\nnn ").unwrap(), ">query\nACGTNN\n");
    }

    #[test]
    fn parse_tabular_hit_uses_domain_region_and_strand() {
        let hit = parse_tabular_hit(
            "query\tchr1\t99.5\t100\t1\t0\t1\t100\t500\t401\t1e-20\t80.0\tACGT\tACGT",
            &AssemblyAccession::new("GCA_test").unwrap(),
        )
        .unwrap();

        assert_eq!(hit.subject_region.start.get(), 401);
        assert_eq!(hit.subject_region.end.get(), 500);
        assert_eq!(hit.sequence_name.as_str(), "chr1");
        assert_eq!(hit.evalue, 1e-20);
    }

    #[test]
    fn validate_task_rejects_arbitrary_arguments() {
        assert!(validate_task("blastn -remote").is_err());
    }
}
