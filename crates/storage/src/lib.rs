use base64::Engine;
use flate2::read::GzDecoder;
use genome_core::{
    Assembly, AssemblyAccession, Exon, Gene, GeneId, GeneRecord, GeneSearch, GenomeDataset,
    GenomeRepository, HalfOpenRegion, Position0, Position1, Sequence, SequenceName, Strand, TaxId,
    Taxon, Transcript, TranscriptId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("domain error: {0}")]
    Domain(#[from] genome_core::DomainError),
    #[error("invalid gff line {line}: expected 9 columns")]
    InvalidGffColumns { line: usize },
    #[error("invalid gff line {line}: {message}")]
    InvalidGffValue { line: usize, message: String },
    #[error("invalid tsv line {line}: {message}")]
    InvalidTsvValue { line: usize, message: String },
    #[error("missing gff attribute {attribute} on line {line}")]
    MissingGffAttribute {
        line: usize,
        attribute: &'static str,
    },
    #[error("missing FASTA sequence for {0}")]
    MissingFastaSequence(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub source_base_url: String,
    pub fasta_file: String,
    pub gff_file: String,
    pub functional_annotation_file: Option<String>,
    pub nomenclature_file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenomeSnapshot {
    pub manifest: SnapshotManifest,
    pub dataset: GenomeDataset,
}

#[derive(Debug, Clone)]
pub struct FileGenomeRepository {
    dataset: GenomeDataset,
    genes_by_id: HashMap<GeneId, Gene>,
    transcripts_by_gene: HashMap<GeneId, Vec<Transcript>>,
    exons_by_transcript: HashMap<TranscriptId, Vec<Exon>>,
    sequence_by_checksum: HashMap<String, Sequence>,
}

impl FileGenomeRepository {
    pub fn new(dataset: GenomeDataset) -> Self {
        let genes_by_id = dataset
            .genes
            .iter()
            .map(|gene| (gene.id.clone(), gene.clone()))
            .collect::<HashMap<_, _>>();

        let mut transcripts_by_gene: HashMap<GeneId, Vec<Transcript>> = HashMap::new();
        for transcript in &dataset.transcripts {
            transcripts_by_gene
                .entry(transcript.gene_id.clone())
                .or_default()
                .push(transcript.clone());
        }

        let mut exons_by_transcript: HashMap<TranscriptId, Vec<Exon>> = HashMap::new();
        for exon in &dataset.exons {
            exons_by_transcript
                .entry(exon.transcript_id.clone())
                .or_default()
                .push(exon.clone());
        }

        let sequence_by_checksum = dataset
            .sequences
            .iter()
            .map(|sequence| (sequence.refget_checksum.clone(), sequence.clone()))
            .collect::<HashMap<_, _>>();

        Self {
            dataset,
            genes_by_id,
            transcripts_by_gene,
            exons_by_transcript,
            sequence_by_checksum,
        }
    }

    pub fn from_snapshot_path(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let snapshot = read_snapshot(path)?;
        Ok(Self::new(snapshot.dataset))
    }

    pub fn default_assembly_accession(&self) -> AssemblyAccession {
        self.dataset.assembly.accession.clone()
    }
}

impl GenomeRepository for FileGenomeRepository {
    fn taxon(&self, tax_id: TaxId) -> Option<Taxon> {
        (self.dataset.taxon.tax_id == tax_id).then(|| self.dataset.taxon.clone())
    }

    fn assembly(&self, accession: &AssemblyAccession) -> Option<Assembly> {
        (&self.dataset.assembly.accession == accession).then(|| self.dataset.assembly.clone())
    }

    fn assemblies_for_taxon(&self, tax_id: TaxId) -> Vec<Assembly> {
        if self.dataset.assembly.tax_id == tax_id {
            vec![self.dataset.assembly.clone()]
        } else {
            Vec::new()
        }
    }

    fn sequences_for_assembly(&self, accession: &AssemblyAccession) -> Vec<Sequence> {
        if &self.dataset.assembly.accession == accession {
            self.dataset.sequences.clone()
        } else {
            Vec::new()
        }
    }

    fn sequence_by_checksum(&self, checksum: &str) -> Option<Sequence> {
        self.sequence_by_checksum.get(checksum).cloned()
    }

    fn gene(&self, gene_id: &GeneId) -> Option<GeneRecord> {
        let gene = self.genes_by_id.get(gene_id)?.clone();
        let transcripts = self
            .transcripts_by_gene
            .get(gene_id)
            .cloned()
            .unwrap_or_default();
        let exons = transcripts
            .iter()
            .flat_map(|transcript| {
                self.exons_by_transcript
                    .get(&transcript.id)
                    .cloned()
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();

        Some(GeneRecord {
            gene,
            transcripts,
            exons,
        })
    }

    fn search_genes(&self, search: &GeneSearch) -> Vec<Gene> {
        let limit = search.limit.unwrap_or(50);
        self.dataset
            .genes
            .iter()
            .filter(|gene| {
                search
                    .tax_id
                    .is_none_or(|tax_id| self.dataset.taxon.tax_id == tax_id)
                    && search_symbol(gene, search.symbol.as_deref())
                    && search_locus_tag(gene, search.locus_tag.as_deref())
                    && search_query(gene, search.query.as_deref())
            })
            .take(limit)
            .cloned()
            .collect()
    }

    fn features_in_region(
        &self,
        accession: &AssemblyAccession,
        region: &HalfOpenRegion,
    ) -> Vec<Gene> {
        if &self.dataset.assembly.accession != accession {
            return Vec::new();
        }

        self.dataset
            .genes
            .iter()
            .filter(|gene| gene.region.overlaps(region))
            .cloned()
            .collect()
    }
}

fn search_symbol(gene: &Gene, symbol: Option<&str>) -> bool {
    symbol.is_none_or(|symbol| {
        gene.symbol
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case(symbol))
    })
}

fn search_locus_tag(gene: &Gene, locus_tag: Option<&str>) -> bool {
    locus_tag.is_none_or(|locus_tag| {
        gene.locus_tag
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case(locus_tag))
    })
}

fn search_query(gene: &Gene, query: Option<&str>) -> bool {
    let Some(query) = query else {
        return true;
    };
    let query = query.to_ascii_lowercase();
    gene.id.as_str().to_ascii_lowercase().contains(&query)
        || gene
            .symbol
            .as_deref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(&query))
        || gene
            .locus_tag
            .as_deref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(&query))
        || gene
            .attributes
            .values()
            .any(|value| value.to_ascii_lowercase().contains(&query))
}

pub fn read_snapshot(path: impl AsRef<Path>) -> Result<GenomeSnapshot, StorageError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

pub fn write_snapshot(
    path: impl AsRef<Path>,
    snapshot: &GenomeSnapshot,
) -> Result<(), StorageError> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, snapshot)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct GenomeSnapshotBuild {
    pub fasta_path: PathBuf,
    pub gff_path: PathBuf,
    pub functional_annotation_path: Option<PathBuf>,
    pub nomenclature_path: Option<PathBuf>,
    pub manifest: SnapshotManifest,
    pub taxon: Taxon,
    pub assembly: Assembly,
}

pub fn build_genome_snapshot(config: &GenomeSnapshotBuild) -> Result<GenomeSnapshot, StorageError> {
    let sequences = read_fasta_sequences(&config.fasta_path)?;
    let mut parsed_gff = parse_gff3(&config.gff_path, &config.assembly.accession)?;
    enrich_parsed_gff(&mut parsed_gff, config)?;

    let sequence_models = sequences
        .values()
        .map(|sequence| {
            Ok(Sequence {
                name: SequenceName::new(sequence.name.clone())?,
                assembly_accession: config.assembly.accession.clone(),
                length: sequence.bases.len() as u64,
                refget_checksum: refget_checksum(sequence.bases.as_bytes()),
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()?;

    let assembly_checksum = assembly_checksum(&sequence_models);
    let mut assembly = config.assembly.clone();
    assembly.refget_checksum = Some(assembly_checksum);

    Ok(GenomeSnapshot {
        manifest: config.manifest.clone(),
        dataset: GenomeDataset {
            taxon: config.taxon.clone(),
            assembly,
            sequences: sequence_models,
            genes: parsed_gff.genes,
            transcripts: parsed_gff.transcripts,
            exons: parsed_gff.exons,
        },
    })
}

#[derive(Debug, Clone)]
pub struct FastaReference {
    by_checksum: HashMap<String, String>,
}

impl FastaReference {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let by_checksum = read_fasta_sequences(path)?
            .into_values()
            .map(|sequence| (refget_checksum(sequence.bases.as_bytes()), sequence.bases))
            .collect::<HashMap<_, _>>();
        Ok(Self { by_checksum })
    }

    pub fn get(&self, checksum: &str, start: Option<u64>, end: Option<u64>) -> Option<String> {
        let sequence = self.by_checksum.get(checksum)?;
        let len = sequence.len() as u64;
        let start = start.unwrap_or(0).min(len);
        let end = end.unwrap_or(len).min(len);
        if start > end {
            return None;
        }
        sequence
            .get(start as usize..end as usize)
            .map(str::to_owned)
    }
}

fn assembly_checksum(sequences: &[Sequence]) -> String {
    let mut checksums = sequences
        .iter()
        .map(|sequence| sequence.refget_checksum.as_str())
        .collect::<Vec<_>>();
    checksums.sort_unstable();

    let mut digest = Sha512::new();
    for checksum in checksums {
        digest.update(checksum.as_bytes());
        digest.update(b"\n");
    }

    let digest = digest.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest[..24])
}

pub fn refget_checksum(sequence: &[u8]) -> String {
    let mut digest = Sha512::new();
    for base in sequence {
        if base.is_ascii_whitespace() {
            continue;
        }
        digest.update([base.to_ascii_uppercase()]);
    }
    let digest = digest.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest[..24])
}

#[derive(Debug, Clone)]
struct FastaSequence {
    name: String,
    bases: String,
}

fn read_fasta_sequences(
    path: impl AsRef<Path>,
) -> Result<HashMap<String, FastaSequence>, StorageError> {
    let path = path.as_ref();
    let reader = open_maybe_gzip(path)?;
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut current_name: Option<String> = None;
    let mut current_bases = String::new();
    let mut sequences = HashMap::new();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        let trimmed = line.trim_end();
        if let Some(header) = trimmed.strip_prefix('>') {
            if let Some(name) = current_name.take() {
                sequences.insert(
                    name.clone(),
                    FastaSequence {
                        name,
                        bases: std::mem::take(&mut current_bases),
                    },
                );
            }
            let name = header
                .split_ascii_whitespace()
                .next()
                .ok_or_else(|| StorageError::MissingFastaSequence(header.to_owned()))?;
            current_name = Some(name.to_owned());
        } else {
            current_bases.push_str(&trimmed.to_ascii_uppercase());
        }
    }

    if let Some(name) = current_name {
        sequences.insert(
            name.clone(),
            FastaSequence {
                name,
                bases: current_bases,
            },
        );
    }

    Ok(sequences)
}

fn open_maybe_gzip(path: &Path) -> Result<Box<dyn Read>, StorageError> {
    let file = File::open(path)?;
    if path.extension().is_some_and(|extension| extension == "gz") {
        Ok(Box::new(GzDecoder::new(file)))
    } else {
        Ok(Box::new(file))
    }
}

#[derive(Debug, Default)]
struct ParsedGff {
    genes: Vec<Gene>,
    transcripts: Vec<Transcript>,
    exons: Vec<Exon>,
}

fn parse_gff3(
    path: impl AsRef<Path>,
    assembly_accession: &AssemblyAccession,
) -> Result<ParsedGff, StorageError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut parsed = ParsedGff::default();
    let mut transcript_parent: HashMap<TranscriptId, GeneId> = HashMap::new();

    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 9 {
            return Err(StorageError::InvalidGffColumns { line: line_number });
        }

        let feature_type = columns[2];
        if feature_type == "gene" || feature_type == "miRNA_gene" {
            parsed
                .genes
                .push(parse_gene(&columns, line_number, assembly_accession)?);
        } else if matches!(feature_type, "mRNA" | "transcript" | "miRNA" | "pre_miRNA") {
            let transcript = parse_transcript(&columns, line_number)?;
            transcript_parent.insert(transcript.id.clone(), transcript.gene_id.clone());
            parsed.transcripts.push(transcript);
        } else if feature_type == "exon" {
            let exons = parse_exons(&columns, line_number, &transcript_parent)?;
            parsed.exons.extend(exons);
        }
    }

    Ok(parsed)
}

fn enrich_parsed_gff(
    parsed: &mut ParsedGff,
    config: &GenomeSnapshotBuild,
) -> Result<(), StorageError> {
    if let Some(path) = &config.functional_annotation_path {
        apply_functional_annotations(parsed, &parse_functional_annotations(path)?);
    }
    if let Some(path) = &config.nomenclature_path {
        apply_nomenclature(parsed, &parse_nomenclature(path)?);
    }

    Ok(())
}

fn apply_functional_annotations(
    parsed: &mut ParsedGff,
    annotations: &HashMap<TranscriptId, String>,
) {
    let mut annotations_by_gene: HashMap<GeneId, Vec<String>> = HashMap::new();

    for transcript in &mut parsed.transcripts {
        let Some(annotation) = annotations.get(&transcript.id) else {
            continue;
        };
        transcript
            .attributes
            .insert("functional_annotation".to_owned(), annotation.clone());
        annotations_by_gene
            .entry(transcript.gene_id.clone())
            .or_default()
            .push(annotation.clone());
    }

    for gene in &mut parsed.genes {
        let Some(values) = annotations_by_gene.get(&gene.id) else {
            continue;
        };
        gene.attributes.insert(
            "functional_annotation".to_owned(),
            join_unique(values.iter().map(String::as_str), " | "),
        );
    }
}

fn parse_functional_annotations(
    path: impl AsRef<Path>,
) -> Result<HashMap<TranscriptId, String>, StorageError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut annotations = HashMap::new();

    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((transcript_id, annotation)) = line.split_once('\t') else {
            return Err(StorageError::InvalidTsvValue {
                line: line_number,
                message: "expected transcript id and annotation columns".to_owned(),
            });
        };
        annotations.insert(TranscriptId::new(transcript_id)?, annotation.to_owned());
    }

    Ok(annotations)
}

#[derive(Debug, Clone, Default)]
struct NomenclatureEntry {
    attributes: BTreeMap<String, String>,
}

fn apply_nomenclature(parsed: &mut ParsedGff, nomenclature: &HashMap<GeneId, NomenclatureEntry>) {
    for gene in &mut parsed.genes {
        let Some(entry) = nomenclature.get(&gene.id) else {
            continue;
        };

        if gene.symbol.is_none() {
            gene.symbol = entry
                .attributes
                .get("nomenclature_symbol")
                .and_then(|value| value.split(" | ").next())
                .map(str::to_owned);
        }
        for (key, value) in &entry.attributes {
            gene.attributes
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }
    }
}

fn parse_nomenclature(
    path: impl AsRef<Path>,
) -> Result<HashMap<GeneId, NomenclatureEntry>, StorageError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let Some(header) = lines.next().transpose()? else {
        return Ok(HashMap::new());
    };
    validate_nomenclature_header(&header)?;

    let mut by_gene = HashMap::new();
    for (index, line) in lines.enumerate() {
        let line_number = index + 2;
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 10 {
            return Err(StorageError::InvalidTsvValue {
                line: line_number,
                message: format!("expected 10 columns, got {}", columns.len()),
            });
        }

        let attributes = nomenclature_attributes(&columns);
        for gene_id in nomenclature_gene_ids(columns[5])? {
            merge_attributes(by_gene.entry(gene_id).or_default(), &attributes);
        }
    }

    Ok(by_gene)
}

fn validate_nomenclature_header(header: &str) -> Result<(), StorageError> {
    let expected = [
        "gene_symbol",
        "full_name",
        "synonym",
        "product",
        "description",
        "GeneID/Location",
        "reference",
        "PMID",
        "DOI",
        "status",
    ];
    let columns = header.split('\t').collect::<Vec<_>>();
    if columns != expected {
        return Err(StorageError::InvalidTsvValue {
            line: 1,
            message: "unexpected nomenclature header".to_owned(),
        });
    }
    Ok(())
}

fn nomenclature_attributes(columns: &[&str]) -> BTreeMap<String, String> {
    [
        ("nomenclature_symbol", columns[0]),
        ("nomenclature_full_name", columns[1]),
        ("nomenclature_synonym", columns[2]),
        ("nomenclature_product", columns[3]),
        ("nomenclature_description", columns[4]),
        ("nomenclature_reference", columns[6]),
        ("nomenclature_pmid", columns[7]),
        ("nomenclature_doi", columns[8]),
        ("nomenclature_status", columns[9]),
    ]
    .into_iter()
    .filter_map(|(key, value)| clean_optional_value(value).map(|value| (key.to_owned(), value)))
    .collect()
}

fn nomenclature_gene_ids(value: &str) -> Result<Vec<GeneId>, StorageError> {
    value
        .split(';')
        .filter_map(clean_gene_reference)
        .map(GeneId::new)
        .collect::<Result<Vec<_>, _>>()
        .map_err(StorageError::from)
}

fn clean_gene_reference(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty()
        || value == "-"
        || value.starts_with("Mapoly")
        || !value.starts_with("Mp")
        || value.contains(':')
    {
        return None;
    }

    Some(strip_transcript_suffix(value))
}

fn strip_transcript_suffix(value: &str) -> String {
    let Some((gene_id, suffix)) = value.rsplit_once('.') else {
        return value.to_owned();
    };
    if suffix.chars().all(|character| character.is_ascii_digit()) {
        gene_id.to_owned()
    } else {
        value.to_owned()
    }
}

fn clean_optional_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value == "-" {
        None
    } else {
        Some(value.to_owned())
    }
}

fn merge_attributes(entry: &mut NomenclatureEntry, attributes: &BTreeMap<String, String>) {
    for (key, value) in attributes {
        match entry.attributes.get_mut(key) {
            Some(existing) => append_unique(existing, value, " | "),
            None => {
                entry.attributes.insert(key.clone(), value.clone());
            }
        }
    }
}

fn append_unique(existing: &mut String, value: &str, separator: &str) {
    if existing.split(separator).any(|part| part == value) {
        return;
    }
    existing.push_str(separator);
    existing.push_str(value);
}

fn join_unique<'a>(values: impl Iterator<Item = &'a str>, separator: &str) -> String {
    let mut joined = String::new();
    for value in values {
        if joined.is_empty() {
            joined.push_str(value);
        } else {
            append_unique(&mut joined, value, separator);
        }
    }
    joined
}

fn parse_gene(
    columns: &[&str],
    line: usize,
    assembly_accession: &AssemblyAccession,
) -> Result<Gene, StorageError> {
    let attrs = parse_attributes(columns[8]);
    let id = attr(&attrs, "ID", line)?;
    let region = parse_region(columns, line)?;

    Ok(Gene {
        id: GeneId::new(id)?,
        assembly_accession: assembly_accession.clone(),
        symbol: pick_attr(&attrs, &["Name", "gene", "symbol"]).filter(|symbol| symbol != id),
        locus_tag: pick_attr(&attrs, &["locus_tag"]),
        sequence_name: SequenceName::new(columns[0])?,
        region,
        strand: Strand::from_str(columns[6])?,
        feature_type: columns[2].to_owned(),
        attributes: attrs,
    })
}

fn parse_transcript(columns: &[&str], line: usize) -> Result<Transcript, StorageError> {
    let attrs = parse_attributes(columns[8]);
    let id = attr(&attrs, "ID", line)?;
    let parent = attr(&attrs, "Parent", line)?;
    let gene_id = parent
        .split(',')
        .next()
        .ok_or(StorageError::MissingGffAttribute {
            line,
            attribute: "Parent",
        })?;

    Ok(Transcript {
        id: TranscriptId::new(id)?,
        gene_id: GeneId::new(gene_id)?,
        sequence_name: SequenceName::new(columns[0])?,
        region: parse_region(columns, line)?,
        strand: Strand::from_str(columns[6])?,
        feature_type: columns[2].to_owned(),
        attributes: attrs,
    })
}

fn parse_exons(
    columns: &[&str],
    line: usize,
    transcript_parent: &HashMap<TranscriptId, GeneId>,
) -> Result<Vec<Exon>, StorageError> {
    let attrs = parse_attributes(columns[8]);
    let parent = attr(&attrs, "Parent", line)?;
    let mut exons = Vec::new();

    for transcript_id in parent.split(',') {
        let transcript_id = TranscriptId::new(transcript_id)?;
        if transcript_parent.contains_key(&transcript_id) {
            exons.push(Exon {
                transcript_id,
                sequence_name: SequenceName::new(columns[0])?,
                region: parse_region(columns, line)?,
                strand: Strand::from_str(columns[6])?,
            });
        }
    }

    Ok(exons)
}

fn parse_region(columns: &[&str], line: usize) -> Result<HalfOpenRegion, StorageError> {
    let start = columns[3]
        .parse::<u64>()
        .map_err(|error| StorageError::InvalidGffValue {
            line,
            message: format!("invalid start: {error}"),
        })?;
    let end = columns[4]
        .parse::<u64>()
        .map_err(|error| StorageError::InvalidGffValue {
            line,
            message: format!("invalid end: {error}"),
        })?;

    let start = Position1::new(start)?.to_position0();
    let end = Position0::new(end);
    Ok(HalfOpenRegion::new(
        SequenceName::new(columns[0])?,
        start,
        end,
    )?)
}

fn parse_attributes(value: &str) -> BTreeMap<String, String> {
    value
        .split(';')
        .filter_map(|part| {
            let (key, value) = part.split_once('=')?;
            Some((key.to_owned(), percent_decode(value)))
        })
        .collect()
}

fn attr<'a>(
    attrs: &'a BTreeMap<String, String>,
    key: &'static str,
    line: usize,
) -> Result<&'a str, StorageError> {
    attrs
        .get(key)
        .map(String::as_str)
        .ok_or(StorageError::MissingGffAttribute {
            line,
            attribute: key,
        })
}

fn pick_attr(attrs: &BTreeMap<String, String>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| attrs.get(*key).cloned())
}

fn percent_decode(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut bytes = value.as_bytes().iter().copied();

    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let Some(high) = bytes.next() else {
                output.push('%');
                break;
            };
            let Some(low) = bytes.next() else {
                output.push('%');
                output.push(high as char);
                break;
            };
            match (hex_value(high), hex_value(low)) {
                (Some(high), Some(low)) => output.push((high * 16 + low) as char),
                _ => {
                    output.push('%');
                    output.push(high as char);
                    output.push(low as char);
                }
            }
        } else {
            output.push(byte as char);
        }
    }

    output
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn refget_checksum_is_case_and_line_invariant() {
        assert_eq!(refget_checksum(b"acgtn\n"), refget_checksum(b"ACGTN"));
    }

    #[test]
    fn snapshot_builder_parses_gene_transcript_and_exon() {
        let dir = tempfile::tempdir().unwrap();
        let fasta_path = dir.path().join("test.fa");
        let gff_path = dir.path().join("test.gff");

        let mut fasta = File::create(&fasta_path).unwrap();
        writeln!(fasta, ">chr1").unwrap();
        writeln!(fasta, "ACGTACGTACGT").unwrap();

        let mut gff = File::create(&gff_path).unwrap();
        writeln!(gff, "##gff-version 3").unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\tgene\t1\t8\t.\t+\t.\tID=Mp1g00010;Name=TEST"
        )
        .unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\tmRNA\t1\t8\t.\t+\t.\tID=Mp1g00010.1;Parent=Mp1g00010"
        )
        .unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\texon\t1\t4\t.\t+\t.\tParent=Mp1g00010.1"
        )
        .unwrap();

        let snapshot = build_genome_snapshot(&GenomeSnapshotBuild {
            fasta_path,
            gff_path,
            manifest: SnapshotManifest {
                source_base_url: "https://example.test".to_owned(),
                fasta_file: "test.fa".to_owned(),
                gff_file: "test.gff".to_owned(),
                functional_annotation_file: None,
                nomenclature_file: None,
            },
            functional_annotation_path: None,
            nomenclature_path: None,
            taxon: Taxon {
                tax_id: genome_core::TaxId::new(3197),
                scientific_name: "Marchantia polymorpha".to_owned(),
                common_name: None,
                rank: "species".to_owned(),
            },
            assembly: Assembly {
                accession: AssemblyAccession::new("GCA_test").unwrap(),
                tax_id: genome_core::TaxId::new(3197),
                name: "test".to_owned(),
                source: genome_core::AssemblySource::Local,
                refget_checksum: None,
            },
        })
        .unwrap();

        assert_eq!(snapshot.dataset.sequences[0].length, 12);
        assert_eq!(
            snapshot.dataset.genes[0].assembly_accession.as_str(),
            "GCA_test"
        );
        assert_eq!(snapshot.dataset.genes[0].id.as_str(), "Mp1g00010");
        assert_eq!(snapshot.dataset.transcripts.len(), 1);
        assert_eq!(snapshot.dataset.exons.len(), 1);
    }

    #[test]
    fn snapshot_builder_imports_functional_annotation_and_nomenclature() {
        let dir = tempfile::tempdir().unwrap();
        let fasta_path = dir.path().join("test.fa");
        let gff_path = dir.path().join("test.gff");
        let functional_annotation_path = dir.path().join("func.tsv");
        let nomenclature_path = dir.path().join("nomenclature.tsv");

        let mut fasta = File::create(&fasta_path).unwrap();
        writeln!(fasta, ">chr1").unwrap();
        writeln!(fasta, "ACGTACGTACGT").unwrap();

        let mut gff = File::create(&gff_path).unwrap();
        writeln!(gff, "##gff-version 3").unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\tgene\t1\t8\t.\t+\t.\tID=Mp1g00010;Name=Mp1g00010"
        )
        .unwrap();
        writeln!(
            gff,
            "chr1\tMarpolBase\tmRNA\t1\t8\t.\t+\t.\tID=Mp1g00010.1;Parent=Mp1g00010"
        )
        .unwrap();

        let mut functional_annotation = File::create(&functional_annotation_path).unwrap();
        writeln!(
            functional_annotation,
            "# This file contains annotation only from KEGG,KOG,Pfam,NCBIfam."
        )
        .unwrap();
        writeln!(
            functional_annotation,
            "Mp1g00010.1\tKEGG:K00001:example annotation; Pfam:PF00001:example domain"
        )
        .unwrap();

        let mut nomenclature = File::create(&nomenclature_path).unwrap();
        writeln!(
            nomenclature,
            "gene_symbol\tfull_name\tsynonym\tproduct\tdescription\tGeneID/Location\treference\tPMID\tDOI\tstatus"
        )
        .unwrap();
        writeln!(
            nomenclature,
            "MpFOO\tFOO FULL\tMpBAR\ttranscription factor\tmanual note\tMapoly0001s0001.1; Mp1g00010.1\tExample reference\t12345\t10.0000/example\tPublished"
        )
        .unwrap();

        let snapshot = build_genome_snapshot(&GenomeSnapshotBuild {
            fasta_path,
            gff_path,
            manifest: SnapshotManifest {
                source_base_url: "https://example.test".to_owned(),
                fasta_file: "test.fa".to_owned(),
                gff_file: "test.gff".to_owned(),
                functional_annotation_file: Some("func.tsv".to_owned()),
                nomenclature_file: Some("nomenclature.tsv".to_owned()),
            },
            functional_annotation_path: Some(functional_annotation_path),
            nomenclature_path: Some(nomenclature_path),
            taxon: Taxon {
                tax_id: genome_core::TaxId::new(3197),
                scientific_name: "Marchantia polymorpha".to_owned(),
                common_name: None,
                rank: "species".to_owned(),
            },
            assembly: Assembly {
                accession: AssemblyAccession::new("GCA_test").unwrap(),
                tax_id: genome_core::TaxId::new(3197),
                name: "test".to_owned(),
                source: genome_core::AssemblySource::Local,
                refget_checksum: None,
            },
        })
        .unwrap();

        let gene = &snapshot.dataset.genes[0];
        let transcript = &snapshot.dataset.transcripts[0];

        assert_eq!(gene.symbol.as_deref(), Some("MpFOO"));
        assert_eq!(
            gene.attributes
                .get("nomenclature_full_name")
                .map(String::as_str),
            Some("FOO FULL")
        );
        assert_eq!(
            gene.attributes
                .get("functional_annotation")
                .map(String::as_str),
            Some("KEGG:K00001:example annotation; Pfam:PF00001:example domain")
        );
        assert_eq!(
            transcript
                .attributes
                .get("functional_annotation")
                .map(String::as_str),
            Some("KEGG:K00001:example annotation; Pfam:PF00001:example domain")
        );
    }
}
