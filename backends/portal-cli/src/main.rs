use clap::{Args, Parser, Subcommand};
use flate2::read::GzDecoder;
use genome_core::{Assembly, AssemblyAccession, AssemblySource, TaxId, Taxon};
use serde::Serialize;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use storage::{GenomeSnapshotBuild, SnapshotManifest, build_genome_snapshot, write_snapshot};
use tracing_subscriber::EnvFilter;

const MARPOLBASE_MPTAK1_V7_1_BASE_URL: &str = "https://marchantia.info/data/MpTak1_v7.1";
const MARPOLBASE_MPTAK1_V7_1_FASTA_FILE: &str = "MpTak1_v7.1.fa.gz";
const MARPOLBASE_MPTAK1_V7_1_GFF_FILE: &str = "MpTak1_v7.1.gff";
const MARPOLBASE_MPTAK1_V7_1_FUNC_ANNOTATION_FILE: &str = "MpTak1_v7.1.func_annotation.1_line.tsv";
const MARPOLBASE_MPTAK1_V7_1_ACCESSION: &str = "GCA_037833805.1";
const MARPOLBASE_NOMENCLATURE_URL: &str = "https://marchantia.info/nomenclature/nomenlatures.txt";
const MARPOLBASE_NOMENCLATURE_FILE: &str = "nomenlatures.txt";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let cli = Cli::parse();
    match cli.command {
        Command::Import(command) => command.run().await?,
        Command::Prepare(command) => command.run()?,
    }

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

#[derive(Debug, Parser)]
#[command(name = "portal-cli")]
#[command(about = "Plant Genome Portal operational CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Import(ImportCommand),
    Prepare(PrepareCommand),
}

#[derive(Debug, Args)]
struct PrepareCommand {
    #[command(subcommand)]
    target: PrepareTarget,
}

impl PrepareCommand {
    fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.target {
            PrepareTarget::Blastn(command) => command.run(),
        }
    }
}

#[derive(Debug, Subcommand)]
enum PrepareTarget {
    /// Prepare a nucleotide BLAST database from a genome FASTA.
    Blastn(BlastnPrepare),
}

#[derive(Debug, Args)]
struct BlastnPrepare {
    #[arg(long)]
    fasta: PathBuf,
    #[arg(long, default_value = "target/blast")]
    out: PathBuf,
    #[arg(long, default_value = "makeblastdb")]
    makeblastdb: PathBuf,
    #[arg(long)]
    manifest: Option<PathBuf>,
}

impl BlastnPrepare {
    fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let prepared = prepare_blastn_database(&self.fasta, &self.out, &self.makeblastdb)?;

        match self.manifest {
            Some(path) => {
                if let Some(parent) = path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                {
                    std::fs::create_dir_all(parent)?;
                }
                serde_json::to_writer_pretty(File::create(path)?, &prepared)?;
            }
            None => {
                let stdout = std::io::stdout();
                let mut stdout = stdout.lock();
                serde_json::to_writer_pretty(&mut stdout, &prepared)?;
                stdout.write_all(b"\n")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreparedBlastDatabase {
    db_prefix: PathBuf,
    fasta: PathBuf,
}

fn prepare_blastn_database(
    fasta: &Path,
    out: &Path,
    makeblastdb: &Path,
) -> Result<PreparedBlastDatabase, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(out)?;
    let fasta = materialize_fasta(fasta, out)?;
    let db_prefix = out.join(database_name(&fasta));
    let output = ProcessCommand::new(makeblastdb)
        .arg("-in")
        .arg(&fasta)
        .arg("-dbtype")
        .arg("nucl")
        .arg("-parse_seqids")
        .arg("-out")
        .arg(&db_prefix)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "{} exited with status {:?}: {}",
            makeblastdb.display(),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(PreparedBlastDatabase { db_prefix, fasta })
}

fn materialize_fasta(fasta: &Path, out: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if fasta.extension().is_none_or(|extension| extension != "gz") {
        return Ok(fasta.to_path_buf());
    }

    let materialized = out.join(format!("{}.fa", database_name(fasta)));
    let mut reader = GzDecoder::new(File::open(fasta)?);
    let mut writer = File::create(&materialized)?;
    io::copy(&mut reader, &mut writer)?;
    Ok(materialized)
}

fn database_name(fasta: &Path) -> String {
    let name_path = fasta
        .extension()
        .filter(|extension| *extension == "gz")
        .and_then(|_| fasta.file_stem())
        .map(Path::new)
        .unwrap_or(fasta);

    name_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("genome")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

#[derive(Debug, Args)]
struct ImportCommand {
    #[command(subcommand)]
    source: ImportSource,
}

impl ImportCommand {
    async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.source {
            ImportSource::MarpolbaseMptak1V7_1(command) => command.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum ImportSource {
    #[command(name = "marpolbase-mptak1-v7-1")]
    MarpolbaseMptak1V7_1(MarpolbaseMptak1V7_1Import),
}

#[derive(Debug, Args)]
struct MarpolbaseMptak1V7_1Import {
    #[arg(long, default_value = "data/marpolbase/MpTak1_v7.1")]
    out: PathBuf,
    #[arg(long)]
    rebuild_snapshot: bool,
    #[arg(long)]
    force: bool,
}

impl MarpolbaseMptak1V7_1Import {
    async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&self.out)?;

        let config = self.config()?;
        if config.snapshot_path.exists()
            && config.input_files_exist()
            && !self.force
            && !self.rebuild_snapshot
        {
            let mut stdout = std::io::stdout().lock();
            writeln!(
                stdout,
                "using existing snapshot {}",
                config.snapshot_path.display()
            )?;
            return Ok(());
        }

        download_if_needed(&config.fasta_url, &config.snapshot.fasta_path, self.force).await?;
        download_if_needed(&config.gff_url, &config.snapshot.gff_path, self.force).await?;
        if let Some(functional_annotation_path) = &config.snapshot.functional_annotation_path {
            download_if_needed(
                &config.functional_annotation_url,
                functional_annotation_path,
                self.force,
            )
            .await?;
        }
        if let Some(nomenclature_path) = &config.snapshot.nomenclature_path {
            download_if_needed(&config.nomenclature_url, nomenclature_path, self.force).await?;
        }

        tracing::info!("parsing MarpolBase Tak-1 v7.1 files");
        let snapshot = build_genome_snapshot(&config.snapshot)?;
        write_snapshot(&config.snapshot_path, &snapshot)?;

        let mut stdout = std::io::stdout().lock();
        writeln!(
            stdout,
            "wrote {} genes, {} transcripts, {} sequences to {}",
            snapshot.dataset.genes.len(),
            snapshot.dataset.transcripts.len(),
            snapshot.dataset.sequences.len(),
            config.snapshot_path.display()
        )?;

        Ok(())
    }

    fn config(&self) -> Result<ImportConfig, Box<dyn std::error::Error>> {
        let fasta_path = self.out.join(MARPOLBASE_MPTAK1_V7_1_FASTA_FILE);
        let gff_path = self.out.join(MARPOLBASE_MPTAK1_V7_1_GFF_FILE);
        let functional_annotation_path = self.out.join(MARPOLBASE_MPTAK1_V7_1_FUNC_ANNOTATION_FILE);
        let nomenclature_path = self.out.join(MARPOLBASE_NOMENCLATURE_FILE);
        let snapshot_path = self.out.join("snapshot.json");
        let tax_id = TaxId::new(3197);

        Ok(ImportConfig {
            fasta_url: format!(
                "{MARPOLBASE_MPTAK1_V7_1_BASE_URL}/{MARPOLBASE_MPTAK1_V7_1_FASTA_FILE}"
            ),
            gff_url: format!("{MARPOLBASE_MPTAK1_V7_1_BASE_URL}/{MARPOLBASE_MPTAK1_V7_1_GFF_FILE}"),
            functional_annotation_url: format!(
                "{MARPOLBASE_MPTAK1_V7_1_BASE_URL}/{MARPOLBASE_MPTAK1_V7_1_FUNC_ANNOTATION_FILE}"
            ),
            nomenclature_url: MARPOLBASE_NOMENCLATURE_URL.to_owned(),
            snapshot_path,
            snapshot: GenomeSnapshotBuild {
                fasta_path,
                gff_path,
                functional_annotation_path: Some(functional_annotation_path),
                nomenclature_path: Some(nomenclature_path),
                manifest: SnapshotManifest {
                    source_base_url: MARPOLBASE_MPTAK1_V7_1_BASE_URL.to_owned(),
                    fasta_file: MARPOLBASE_MPTAK1_V7_1_FASTA_FILE.to_owned(),
                    gff_file: MARPOLBASE_MPTAK1_V7_1_GFF_FILE.to_owned(),
                    functional_annotation_file: Some(
                        MARPOLBASE_MPTAK1_V7_1_FUNC_ANNOTATION_FILE.to_owned(),
                    ),
                    nomenclature_file: Some(MARPOLBASE_NOMENCLATURE_FILE.to_owned()),
                },
                taxon: Taxon {
                    tax_id,
                    scientific_name: "Marchantia polymorpha".to_owned(),
                    common_name: Some("common liverwort".to_owned()),
                    rank: "species".to_owned(),
                },
                assembly: Assembly {
                    accession: AssemblyAccession::new(MARPOLBASE_MPTAK1_V7_1_ACCESSION)?,
                    tax_id,
                    name: "MpTak1_v7.1".to_owned(),
                    source: AssemblySource::MarpolBase,
                    refget_checksum: None,
                },
            },
        })
    }
}

#[derive(Debug)]
struct ImportConfig {
    fasta_url: String,
    gff_url: String,
    functional_annotation_url: String,
    nomenclature_url: String,
    snapshot_path: PathBuf,
    snapshot: GenomeSnapshotBuild,
}

impl ImportConfig {
    fn input_files_exist(&self) -> bool {
        self.snapshot.fasta_path.exists()
            && self.snapshot.gff_path.exists()
            && self
                .snapshot
                .functional_annotation_path
                .as_ref()
                .is_none_or(|path| path.exists())
            && self
                .snapshot
                .nomenclature_path
                .as_ref()
                .is_none_or(|path| path.exists())
    }
}

async fn download_if_needed(
    url: &str,
    path: &Path,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() && !force {
        tracing::info!(path = %path.display(), "using existing file");
        return Ok(());
    }

    tracing::info!(url, path = %path.display(), "downloading");
    let bytes = reqwest::get(url).await?.error_for_status()?.bytes().await?;
    let mut file = std::fs::File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}
