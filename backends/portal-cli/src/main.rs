use clap::{Args, Parser, Subcommand};
use epigenome_core::Experiment;
use epigenome_store::{
    EpigenomeDataset, EpigenomeSnapshot, EpigenomeSnapshotManifest, ExperimentPeaks,
    parsers::{
        ExperimentManifestEntry, open_peaks, parse_broad_peak, parse_manifest, parse_narrow_peak,
    },
    write_snapshot as write_epigenome_snapshot,
};
use flate2::read::GzDecoder;
use genome_core::{Assembly, AssemblyAccession, AssemblySource, TaxId, Taxon};
use serde::Serialize;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use storage::{
    GenomeSnapshot, GenomeSnapshotBuild, KeggCatalogPaths, KeggManifest, SnapshotManifest,
    build_genome_snapshot, write_snapshot,
};
use tracing_subscriber::EnvFilter;

const MARPOLBASE_MPTAK1_V7_1_BASE_URL: &str = "https://marchantia.info/data/MpTak1_v7.1";
const MARPOLBASE_MPTAK1_V7_1_FASTA_FILE: &str = "MpTak1_v7.1.fa.gz";
const MARPOLBASE_MPTAK1_V7_1_GFF_FILE: &str = "MpTak1_v7.1.gff";
const MARPOLBASE_MPTAK1_V7_1_FUNC_ANNOTATION_FILE: &str = "MpTak1_v7.1.func_annotation.1_line.tsv";
const MARPOLBASE_MPTAK1_V7_1_PROTEIN_FILE: &str = "MpTak1_v7.1.protein.fa";
const MARPOLBASE_MPTAK1_V7_1_ACCESSION: &str = "GCA_037833805.1";
const MARPOLBASE_NOMENCLATURE_URL: &str = "https://marchantia.info/nomenclature/nomenlatures.txt";
const MARPOLBASE_NOMENCLATURE_FILE: &str = "nomenlatures.txt";

const KEGG_REST_BASE_URL: &str = "https://rest.kegg.jp";
const KEGG_LINK_KO_PATHWAY_FILE: &str = "kegg.link.ko-pathway.tsv";
const KEGG_LINK_KO_MODULE_FILE: &str = "kegg.link.ko-module.tsv";
const KEGG_LINK_KO_REACTION_FILE: &str = "kegg.link.ko-reaction.tsv";
const KEGG_LIST_PATHWAY_FILE: &str = "kegg.list.pathway.tsv";
const KEGG_LIST_MODULE_FILE: &str = "kegg.list.module.tsv";
const KEGG_LIST_REACTION_FILE: &str = "kegg.list.reaction.tsv";

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
            PrepareTarget::Blastn(command) => command.run("nucl"),
            PrepareTarget::Blastp(command) => command.run("prot"),
        }
    }
}

#[derive(Debug, Subcommand)]
enum PrepareTarget {
    /// Prepare a nucleotide BLAST database from a genome FASTA.
    Blastn(BlastDatabasePrepare),
    /// Prepare a protein BLAST database from a protein FASTA.
    Blastp(BlastDatabasePrepare),
}

#[derive(Debug, Args)]
struct BlastDatabasePrepare {
    #[arg(long)]
    fasta: PathBuf,
    #[arg(long, default_value = "target/blast")]
    out: PathBuf,
    #[arg(long, default_value = "makeblastdb")]
    makeblastdb: PathBuf,
    #[arg(long)]
    manifest: Option<PathBuf>,
}

impl BlastDatabasePrepare {
    fn run(self, dbtype: &str) -> Result<(), Box<dyn std::error::Error>> {
        let prepared = prepare_blast_database(&self.fasta, &self.out, &self.makeblastdb, dbtype)?;

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

fn prepare_blast_database(
    fasta: &Path,
    out: &Path,
    makeblastdb: &Path,
    dbtype: &str,
) -> Result<PreparedBlastDatabase, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(out)?;
    let fasta = materialize_fasta(fasta, out)?;
    let db_prefix = out.join(database_name(&fasta));
    let output = ProcessCommand::new(makeblastdb)
        .arg("-in")
        .arg(&fasta)
        .arg("-dbtype")
        .arg(dbtype)
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
            ImportSource::EpigenomeManifest(command) => command.run(),
        }
    }
}

#[derive(Debug, Subcommand)]
enum ImportSource {
    #[command(name = "marpolbase-mptak1-v7-1")]
    MarpolbaseMptak1V7_1(MarpolbaseMptak1V7_1Import),
    /// Import ChIP-seq / ATAC-seq peaks and metadata from a curator-written
    /// TOML manifest, producing an `epigenome_snapshot.json` consumed by the
    /// API server via `--epigenome-snapshot`.
    #[command(name = "epigenome-manifest")]
    EpigenomeManifest(EpigenomeManifestImport),
}

#[derive(Debug, Args)]
struct EpigenomeManifestImport {
    /// Path to the curator-facing TOML manifest. See
    /// `crates/epigenome-store/src/parsers/manifest.rs` for the schema.
    #[arg(long)]
    manifest: PathBuf,
    /// Output path for the resulting `epigenome_snapshot.json`.
    #[arg(long)]
    out: PathBuf,
    /// Free-form source label written into the snapshot manifest (e.g.
    /// `"marpolbase-curated-2026-04"`).
    #[arg(long, default_value = "curator-manifest")]
    source_label: String,
    #[arg(long)]
    description: Option<String>,
}

impl EpigenomeManifestImport {
    fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let entries = parse_manifest(&self.manifest)?;
        let manifest_dir = self.manifest.parent().unwrap_or_else(|| Path::new("."));

        let assembly = entries[0].assembly_accession.clone();
        for entry in &entries {
            if entry.assembly_accession != assembly {
                return Err(format!(
                    "manifest mixes assemblies: {} vs {} in experiment {}",
                    assembly, entry.assembly_accession, entry.id
                )
                .into());
            }
        }

        let mut experiments: Vec<Experiment> = Vec::with_capacity(entries.len());
        let mut peaks_groups: Vec<ExperimentPeaks> = Vec::with_capacity(entries.len());

        for entry in &entries {
            let peak_path = entry.peak_path(manifest_dir);
            tracing::info!(
                experiment = %entry.id,
                peak_file = %peak_path.display(),
                "parsing peak file"
            );
            let reader = open_peaks(&peak_path)?;
            let peaks = match entry.peak_kind {
                epigenome_core::PeakKind::Narrow => parse_narrow_peak(reader)?,
                epigenome_core::PeakKind::Broad => parse_broad_peak(reader)?,
            };

            let signal_file = entry.signal_path(manifest_dir).as_ref().and_then(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_owned)
            });

            experiments.push(build_experiment(entry, signal_file));
            peaks_groups.push(ExperimentPeaks {
                experiment_id: entry.id.clone(),
                kind: entry.peak_kind,
                peaks,
            });
        }

        let snapshot = EpigenomeSnapshot {
            manifest: EpigenomeSnapshotManifest {
                source: self.source_label,
                description: self.description,
            },
            dataset: EpigenomeDataset {
                assembly_accession: assembly,
                experiments,
                peaks: peaks_groups,
            },
        };

        if let Some(parent) = self
            .out
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        write_epigenome_snapshot(&self.out, &snapshot)?;

        let mut stdout = std::io::stdout().lock();
        writeln!(
            stdout,
            "wrote {} experiments ({} peak groups) to {}",
            snapshot.dataset.experiments.len(),
            snapshot.dataset.peaks.len(),
            self.out.display()
        )?;
        Ok(())
    }
}

fn build_experiment(entry: &ExperimentManifestEntry, signal_file: Option<String>) -> Experiment {
    Experiment {
        id: entry.id.clone(),
        assay: entry.assay,
        target: entry.target.clone(),
        antibody: entry.antibody.clone(),
        assembly_accession: entry.assembly_accession.clone(),
        geo_series: entry.geo_series.clone(),
        geo_sample: entry.geo_sample.clone(),
        sra_runs: entry.sra_runs.clone(),
        tissue: entry.tissue.clone(),
        dev_stage: entry.dev_stage.clone(),
        treatment: entry.treatment.clone(),
        replicate: entry.replicate,
        pipeline: entry.pipeline.clone(),
        qvalue_cutoff: entry.qvalue_cutoff,
        qc: epigenome_core::ExperimentQc {
            frip: entry.qc.frip,
            nrf: entry.qc.nrf,
            nsc: entry.qc.nsc,
            rsc: entry.qc.rsc,
            mapped_reads: entry.qc.mapped_reads,
        },
        peak_kind: entry.peak_kind,
        signal_file,
        attributes: entry.attributes.clone(),
    }
}

#[derive(Debug, Args)]
struct MarpolbaseMptak1V7_1Import {
    #[arg(long, default_value = "data/marpolbase/MpTak1_v7.1")]
    out: PathBuf,
    /// Directory to cache KEGG REST dumps shared across imports.
    #[arg(long, default_value = "data/kegg")]
    kegg_dir: PathBuf,
    /// Skip downloading the KEGG cross-link catalog (useful for offline builds).
    #[arg(long)]
    skip_kegg: bool,
    #[arg(long)]
    rebuild_snapshot: bool,
    #[arg(long)]
    force: bool,
}

impl MarpolbaseMptak1V7_1Import {
    async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&self.out)?;

        let config = self.config()?;
        if self.use_existing_snapshot(&config)? {
            return Ok(());
        }

        self.download_inputs(&config).await?;

        tracing::info!("parsing MarpolBase Tak-1 v7.1 files");
        let snapshot = build_genome_snapshot(&config.snapshot)?;
        write_snapshot(&config.snapshot_path, &snapshot)?;
        write_import_summary(&snapshot, &config.snapshot_path)?;

        Ok(())
    }

    fn use_existing_snapshot(
        &self,
        config: &ImportConfig,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if !config.snapshot_path.exists()
            || !config.input_files_exist()
            || self.force
            || self.rebuild_snapshot
        {
            return Ok(false);
        }
        let mut stdout = std::io::stdout().lock();
        writeln!(
            stdout,
            "using existing snapshot {}",
            config.snapshot_path.display()
        )?;
        Ok(true)
    }

    async fn download_inputs(
        &self,
        config: &ImportConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        if let Some(protein_fasta_path) = &config.snapshot.protein_fasta_path {
            download_if_needed(&config.protein_fasta_url, protein_fasta_path, self.force).await?;
        }
        for download in &config.kegg_downloads {
            download_if_needed(&download.url, &download.path, self.force).await?;
        }
        Ok(())
    }

    fn config(&self) -> Result<ImportConfig, Box<dyn std::error::Error>> {
        let fasta_path = self.out.join(MARPOLBASE_MPTAK1_V7_1_FASTA_FILE);
        let gff_path = self.out.join(MARPOLBASE_MPTAK1_V7_1_GFF_FILE);
        let functional_annotation_path = self.out.join(MARPOLBASE_MPTAK1_V7_1_FUNC_ANNOTATION_FILE);
        let nomenclature_path = self.out.join(MARPOLBASE_NOMENCLATURE_FILE);
        let protein_fasta_path = self.out.join(MARPOLBASE_MPTAK1_V7_1_PROTEIN_FILE);
        let snapshot_path = self.out.join("snapshot.json");
        let tax_id = TaxId::new(3197);

        let layout = if self.skip_kegg {
            None
        } else {
            std::fs::create_dir_all(&self.kegg_dir)?;
            Some(kegg_dump_layout(&self.kegg_dir)?)
        };
        let kegg_catalog_paths = layout
            .as_ref()
            .map(|layout| layout.catalog_paths.clone())
            .unwrap_or_default();
        let kegg_downloads = layout
            .as_ref()
            .map(|layout| layout.downloads.clone())
            .unwrap_or_default();
        let kegg_manifest = layout.map(|layout| layout.manifest);

        Ok(ImportConfig {
            fasta_url: format!(
                "{MARPOLBASE_MPTAK1_V7_1_BASE_URL}/{MARPOLBASE_MPTAK1_V7_1_FASTA_FILE}"
            ),
            gff_url: format!("{MARPOLBASE_MPTAK1_V7_1_BASE_URL}/{MARPOLBASE_MPTAK1_V7_1_GFF_FILE}"),
            functional_annotation_url: format!(
                "{MARPOLBASE_MPTAK1_V7_1_BASE_URL}/{MARPOLBASE_MPTAK1_V7_1_FUNC_ANNOTATION_FILE}"
            ),
            nomenclature_url: MARPOLBASE_NOMENCLATURE_URL.to_owned(),
            protein_fasta_url: format!(
                "{MARPOLBASE_MPTAK1_V7_1_BASE_URL}/{MARPOLBASE_MPTAK1_V7_1_PROTEIN_FILE}"
            ),
            snapshot_path,
            kegg_downloads,
            snapshot: GenomeSnapshotBuild {
                fasta_path,
                gff_path,
                functional_annotation_path: Some(functional_annotation_path),
                nomenclature_path: Some(nomenclature_path),
                protein_fasta_path: Some(protein_fasta_path),
                kegg_catalog_paths,
                manifest: SnapshotManifest {
                    source_base_url: MARPOLBASE_MPTAK1_V7_1_BASE_URL.to_owned(),
                    fasta_file: MARPOLBASE_MPTAK1_V7_1_FASTA_FILE.to_owned(),
                    gff_file: MARPOLBASE_MPTAK1_V7_1_GFF_FILE.to_owned(),
                    functional_annotation_file: Some(
                        MARPOLBASE_MPTAK1_V7_1_FUNC_ANNOTATION_FILE.to_owned(),
                    ),
                    nomenclature_file: Some(MARPOLBASE_NOMENCLATURE_FILE.to_owned()),
                    kegg_files: kegg_manifest,
                    protein_fasta_file: Some(MARPOLBASE_MPTAK1_V7_1_PROTEIN_FILE.to_owned()),
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

#[derive(Debug, Clone)]
struct KeggDownload {
    url: String,
    path: PathBuf,
}

#[derive(Debug, Clone)]
struct KeggDumpLayout {
    catalog_paths: KeggCatalogPaths,
    downloads: Vec<KeggDownload>,
    manifest: KeggManifest,
}

fn kegg_dump_layout(dir: &Path) -> Result<KeggDumpLayout, Box<dyn std::error::Error>> {
    let link_ko_pathway = dir.join(KEGG_LINK_KO_PATHWAY_FILE);
    let link_ko_module = dir.join(KEGG_LINK_KO_MODULE_FILE);
    let link_ko_reaction = dir.join(KEGG_LINK_KO_REACTION_FILE);
    let list_pathway = dir.join(KEGG_LIST_PATHWAY_FILE);
    let list_module = dir.join(KEGG_LIST_MODULE_FILE);
    let list_reaction = dir.join(KEGG_LIST_REACTION_FILE);

    let downloads = vec![
        KeggDownload {
            url: format!("{KEGG_REST_BASE_URL}/link/pathway/ko"),
            path: link_ko_pathway.clone(),
        },
        KeggDownload {
            url: format!("{KEGG_REST_BASE_URL}/link/module/ko"),
            path: link_ko_module.clone(),
        },
        KeggDownload {
            url: format!("{KEGG_REST_BASE_URL}/link/reaction/ko"),
            path: link_ko_reaction.clone(),
        },
        KeggDownload {
            url: format!("{KEGG_REST_BASE_URL}/list/pathway"),
            path: list_pathway.clone(),
        },
        KeggDownload {
            url: format!("{KEGG_REST_BASE_URL}/list/module"),
            path: list_module.clone(),
        },
        KeggDownload {
            url: format!("{KEGG_REST_BASE_URL}/list/reaction"),
            path: list_reaction.clone(),
        },
    ];

    let manifest = KeggManifest {
        source_base_url: KEGG_REST_BASE_URL.to_owned(),
        link_ko_pathway: Some(KEGG_LINK_KO_PATHWAY_FILE.to_owned()),
        link_ko_module: Some(KEGG_LINK_KO_MODULE_FILE.to_owned()),
        link_ko_reaction: Some(KEGG_LINK_KO_REACTION_FILE.to_owned()),
        list_pathway: Some(KEGG_LIST_PATHWAY_FILE.to_owned()),
        list_module: Some(KEGG_LIST_MODULE_FILE.to_owned()),
        list_reaction: Some(KEGG_LIST_REACTION_FILE.to_owned()),
    };

    Ok(KeggDumpLayout {
        catalog_paths: KeggCatalogPaths {
            link_ko_pathway: Some(link_ko_pathway),
            link_ko_module: Some(link_ko_module),
            link_ko_reaction: Some(link_ko_reaction),
            list_pathway: Some(list_pathway),
            list_module: Some(list_module),
            list_reaction: Some(list_reaction),
        },
        downloads,
        manifest,
    })
}

fn write_import_summary(
    snapshot: &GenomeSnapshot,
    snapshot_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout().lock();
    writeln!(
        stdout,
        "wrote {} genes, {} transcripts, {} sequences to {}",
        snapshot.dataset.genes.len(),
        snapshot.dataset.transcripts.len(),
        snapshot.dataset.sequences.len(),
        snapshot_path.display()
    )?;
    Ok(())
}

#[derive(Debug)]
struct ImportConfig {
    fasta_url: String,
    gff_url: String,
    functional_annotation_url: String,
    nomenclature_url: String,
    protein_fasta_url: String,
    snapshot_path: PathBuf,
    kegg_downloads: Vec<KeggDownload>,
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
            && self
                .snapshot
                .protein_fasta_path
                .as_ref()
                .is_none_or(|path| path.exists())
            && self
                .kegg_downloads
                .iter()
                .all(|download| download.path.exists())
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
