mod blast;
mod codec;

use blast::{BlastHomologySearchInput, BlastRunner};
use clap::{Parser, Subcommand};
use codec::{JobCodec, MessagePack};
use genome_core::AssemblyAccession;
use service::{Worker, WorkerJob};
use std::fs::{self, File};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "worker")]
#[command(about = "Plant Genome Portal background worker")]
struct Config {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run one BLASTN homology search and write a domain-normalized JSON result.
    BlastnOnce {
        #[arg(long)]
        assembly_accession: AssemblyAccession,
        #[arg(long)]
        blast_db_prefix: PathBuf,
        #[arg(long, default_value = "target/worker")]
        work_dir: PathBuf,
        #[arg(long, default_value = "blastn")]
        blastn: PathBuf,
        #[arg(long)]
        query: String,
        #[arg(long)]
        snapshot: Option<PathBuf>,
        #[arg(long, default_value = "blastn")]
        task: String,
        #[arg(long, default_value_t = 10.0)]
        evalue: f64,
        #[arg(long, default_value_t = 50)]
        max_target_seqs: usize,
        #[arg(long)]
        output: PathBuf,
    },
    /// Run one BLASTN homology search from a MessagePack WorkerJob and write a MessagePack result.
    BlastnJob {
        #[arg(long)]
        blast_db_prefix: PathBuf,
        #[arg(long, default_value = "target/worker")]
        work_dir: PathBuf,
        #[arg(long, default_value = "blastn")]
        blastn: PathBuf,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Run one BLASTP homology search and write a domain-normalized JSON result.
    BlastpOnce {
        #[arg(long)]
        assembly_accession: AssemblyAccession,
        #[arg(long)]
        blast_db_prefix: PathBuf,
        #[arg(long, default_value = "target/worker")]
        work_dir: PathBuf,
        #[arg(long, default_value = "blastp")]
        blastp: PathBuf,
        #[arg(long)]
        query: String,
        #[arg(long)]
        snapshot: Option<PathBuf>,
        #[arg(long, default_value = "blastp")]
        task: String,
        #[arg(long, default_value_t = 10.0)]
        evalue: f64,
        #[arg(long, default_value_t = 50)]
        max_target_seqs: usize,
        #[arg(long)]
        output: PathBuf,
    },
    /// Run one BLASTP homology search from a MessagePack WorkerJob and write a MessagePack result.
    BlastpJob {
        #[arg(long)]
        blast_db_prefix: PathBuf,
        #[arg(long, default_value = "target/worker")]
        work_dir: PathBuf,
        #[arg(long, default_value = "blastp")]
        blastp: PathBuf,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();

    match config.command {
        Command::BlastnOnce {
            assembly_accession,
            blast_db_prefix,
            work_dir,
            blastn,
            query,
            snapshot,
            task,
            evalue,
            max_target_seqs,
            output,
        } => {
            let runner = BlastRunner::blastn(blast_db_prefix, work_dir, blastn)?;
            let payload = BlastHomologySearchInput {
                assembly_accession,
                query,
                task,
                evalue,
                max_target_seqs,
                snapshot,
            };
            let job = WorkerJob {
                id: "blastn-once".to_owned(),
                kind: "homology.blastn".to_owned(),
                payload,
            };
            let result = runner.run(job)?;
            serde_json::to_writer_pretty(File::create(output)?, &result)?;
        }
        Command::BlastnJob {
            blast_db_prefix,
            work_dir,
            blastn,
            input,
            output,
        } => {
            let runner = BlastRunner::blastn(blast_db_prefix, work_dir, blastn)?;
            let bytes = fs::read(input)?;
            let job = MessagePack::<WorkerJob<BlastHomologySearchInput>>::decode(&bytes)?;
            let result = runner.run(job)?;
            let bytes = MessagePack::encode(&result)?;
            fs::write(output, bytes)?;
        }
        Command::BlastpOnce {
            assembly_accession,
            blast_db_prefix,
            work_dir,
            blastp,
            query,
            snapshot,
            task,
            evalue,
            max_target_seqs,
            output,
        } => {
            let runner = BlastRunner::blastp(blast_db_prefix, work_dir, blastp)?;
            let payload = BlastHomologySearchInput {
                assembly_accession,
                query,
                task,
                evalue,
                max_target_seqs,
                snapshot,
            };
            let job = WorkerJob {
                id: "blastp-once".to_owned(),
                kind: "homology.blastp".to_owned(),
                payload,
            };
            let result = runner.run(job)?;
            serde_json::to_writer_pretty(File::create(output)?, &result)?;
        }
        Command::BlastpJob {
            blast_db_prefix,
            work_dir,
            blastp,
            input,
            output,
        } => {
            let runner = BlastRunner::blastp(blast_db_prefix, work_dir, blastp)?;
            let bytes = fs::read(input)?;
            let job = MessagePack::<WorkerJob<BlastHomologySearchInput>>::decode(&bytes)?;
            let result = runner.run(job)?;
            let bytes = MessagePack::encode(&result)?;
            fs::write(output, bytes)?;
        }
    }

    Ok(())
}
