use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use genome_core::{
    AssemblyAccession, Cds, Exon, Gene, GeneId, HalfOpenRegion, Position0, Position1, SequenceName,
    Strand, Transcript, TranscriptId,
};
use noodles_gff as gff;
use noodles_gff::feature::record::Phase;
use noodles_gff::feature::record_buf::Attributes;

use crate::error::StorageError;

#[derive(Debug, Default)]
pub(crate) struct ParsedGff {
    pub genes: Vec<Gene>,
    pub transcripts: Vec<Transcript>,
    pub exons: Vec<Exon>,
    pub cdss: Vec<Cds>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GffFeatureKind {
    Gene,
    Transcript,
    Exon,
    Cds,
    Ignored,
}

impl GffFeatureKind {
    fn from_type(feature_type: &str) -> Self {
        match feature_type {
            "gene" | "miRNA_gene" => Self::Gene,
            "mRNA" | "transcript" | "miRNA" | "pre_miRNA" => Self::Transcript,
            "exon" => Self::Exon,
            "CDS" => Self::Cds,
            _ => Self::Ignored,
        }
    }
}

pub(crate) fn parse_gff3(
    path: impl AsRef<Path>,
    assembly_accession: &AssemblyAccession,
) -> Result<ParsedGff, StorageError> {
    let file = File::open(path)?;
    let mut reader = gff::io::Reader::new(BufReader::new(file));

    let mut parsed = ParsedGff::default();
    let mut transcript_parent: HashMap<TranscriptId, GeneId> = HashMap::new();

    for (index, result) in reader.record_bufs().enumerate() {
        let record = result.map_err(|error| StorageError::InvalidGffValue {
            line: index + 1,
            message: error.to_string(),
        })?;
        let line_number = index + 1;
        let feature_type = bytes_to_string(record.ty().as_ref());

        match GffFeatureKind::from_type(&feature_type) {
            GffFeatureKind::Gene => {
                parsed
                    .genes
                    .push(parse_gene(&record, line_number, assembly_accession)?);
            }
            GffFeatureKind::Transcript => {
                let transcript = parse_transcript(&record, line_number)?;
                transcript_parent.insert(transcript.id.clone(), transcript.gene_id.clone());
                parsed.transcripts.push(transcript);
            }
            GffFeatureKind::Exon => {
                let exons = parse_exons(&record, line_number, &transcript_parent)?;
                parsed.exons.extend(exons);
            }
            GffFeatureKind::Cds => {
                let cdss = parse_cdss(&record, line_number, &transcript_parent)?;
                parsed.cdss.extend(cdss);
            }
            GffFeatureKind::Ignored => {}
        }
    }

    Ok(parsed)
}

fn parse_gene(
    record: &gff::feature::RecordBuf,
    line: usize,
    assembly_accession: &AssemblyAccession,
) -> Result<Gene, StorageError> {
    let attrs = collect_attributes(record.attributes());
    let id = attr(&attrs, "ID", line)?;
    let region = parse_region(record, line)?;
    let sequence_name = SequenceName::new(bytes_to_string(record.reference_sequence_name()))?;
    let feature_type = bytes_to_string(record.ty().as_ref());

    Ok(Gene {
        id: GeneId::new(id)?,
        assembly_accession: assembly_accession.clone(),
        symbol: pick_attr(&attrs, &["Name", "gene", "symbol"]).filter(|symbol| symbol != id),
        locus_tag: pick_attr(&attrs, &["locus_tag"]),
        sequence_name,
        region,
        strand: convert_strand(record.strand()),
        feature_type,
        annotations: Vec::new(),
        attributes: attrs,
    })
}

fn parse_transcript(
    record: &gff::feature::RecordBuf,
    line: usize,
) -> Result<Transcript, StorageError> {
    let attrs = collect_attributes(record.attributes());
    let id = attr(&attrs, "ID", line)?;
    let parent = attr(&attrs, "Parent", line)?;
    let gene_id = parent
        .split(',')
        .next()
        .ok_or(StorageError::MissingGffAttribute {
            line,
            attribute: "Parent",
        })?;
    let sequence_name = SequenceName::new(bytes_to_string(record.reference_sequence_name()))?;
    let feature_type = bytes_to_string(record.ty().as_ref());

    Ok(Transcript {
        id: TranscriptId::new(id)?,
        gene_id: GeneId::new(gene_id)?,
        sequence_name,
        region: parse_region(record, line)?,
        strand: convert_strand(record.strand()),
        feature_type,
        annotations: Vec::new(),
        attributes: attrs,
        protein_checksum: None,
        protein_length: None,
    })
}

fn parse_exons(
    record: &gff::feature::RecordBuf,
    line: usize,
    transcript_parent: &HashMap<TranscriptId, GeneId>,
) -> Result<Vec<Exon>, StorageError> {
    let attrs = collect_attributes(record.attributes());
    let parent = attr(&attrs, "Parent", line)?;
    let sequence_name = SequenceName::new(bytes_to_string(record.reference_sequence_name()))?;
    let strand = convert_strand(record.strand());
    let region = parse_region(record, line)?;
    let mut exons = Vec::new();

    for transcript_id in parent.split(',') {
        let transcript_id = TranscriptId::new(transcript_id)?;
        if transcript_parent.contains_key(&transcript_id) {
            exons.push(Exon {
                transcript_id,
                sequence_name: sequence_name.clone(),
                region: region.clone(),
                strand,
            });
        }
    }

    Ok(exons)
}

fn parse_cdss(
    record: &gff::feature::RecordBuf,
    line: usize,
    transcript_parent: &HashMap<TranscriptId, GeneId>,
) -> Result<Vec<Cds>, StorageError> {
    let attrs = collect_attributes(record.attributes());
    let parent = attr(&attrs, "Parent", line)?;
    let sequence_name = SequenceName::new(bytes_to_string(record.reference_sequence_name()))?;
    let strand = convert_strand(record.strand());
    let region = parse_region(record, line)?;
    let phase = record.phase().map(convert_phase);
    let mut cdss = Vec::new();

    for transcript_id in parent.split(',') {
        let transcript_id = TranscriptId::new(transcript_id)?;
        if transcript_parent.contains_key(&transcript_id) {
            cdss.push(Cds {
                transcript_id,
                sequence_name: sequence_name.clone(),
                region: region.clone(),
                strand,
                phase,
            });
        }
    }

    Ok(cdss)
}

fn convert_phase(phase: Phase) -> u8 {
    match phase {
        Phase::Zero => 0,
        Phase::One => 1,
        Phase::Two => 2,
    }
}

fn parse_region(
    record: &gff::feature::RecordBuf,
    line: usize,
) -> Result<HalfOpenRegion, StorageError> {
    let start = record.start().get() as u64;
    let end = record.end().get() as u64;

    let start = Position1::new(start)?.to_position0();
    let end = Position0::new(end);
    let sequence_name = SequenceName::new(bytes_to_string(record.reference_sequence_name()))?;
    HalfOpenRegion::new(sequence_name, start, end).map_err(|error| StorageError::InvalidGffValue {
        line,
        message: error.to_string(),
    })
}

fn convert_strand(strand: gff::feature::record::Strand) -> Strand {
    match strand {
        gff::feature::record::Strand::Forward => Strand::Forward,
        gff::feature::record::Strand::Reverse => Strand::Reverse,
        gff::feature::record::Strand::None | gff::feature::record::Strand::Unknown => {
            Strand::Unknown
        }
    }
}

fn collect_attributes(attributes: &Attributes) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (tag, value) in attributes.as_ref() {
        let key = bytes_to_string(tag.as_ref());
        let value_string = match value {
            noodles_gff::feature::record_buf::attributes::field::Value::String(value) => {
                bytes_to_string(value.as_ref())
            }
            noodles_gff::feature::record_buf::attributes::field::Value::Array(values) => values
                .iter()
                .map(|value| bytes_to_string(value.as_ref()))
                .collect::<Vec<_>>()
                .join(","),
        };
        out.insert(key, value_string);
    }
    out
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

fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn pick_attr_returns_first_matching_key() {
        let mut attrs = BTreeMap::new();
        attrs.insert("Name".to_owned(), "primary".to_owned());
        attrs.insert("symbol".to_owned(), "fallback".to_owned());
        assert_eq!(
            pick_attr(&attrs, &["Name", "symbol"]),
            Some("primary".to_owned())
        );
    }

    #[test]
    fn pick_attr_falls_through_to_next_key() {
        let mut attrs = BTreeMap::new();
        attrs.insert("symbol".to_owned(), "fallback".to_owned());
        assert_eq!(
            pick_attr(&attrs, &["Name", "symbol"]),
            Some("fallback".to_owned())
        );
    }

    #[test]
    fn pick_attr_returns_none_when_no_key_matches() {
        let attrs = BTreeMap::new();
        assert_eq!(pick_attr(&attrs, &["Name", "symbol"]), None);
    }

    #[test]
    fn parse_gff3_reports_line_number_on_noodles_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("invalid.gff");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "##gff-version 3").unwrap();
        writeln!(file, "chr1\tsrc\tgene\t1\t8\t.\t+\t.\tID=Mp1g00010;Name=ok").unwrap();
        // Bad start coordinate triggers parse error on the second record.
        writeln!(
            file,
            "chr1\tsrc\tgene\tNOT_A_NUMBER\t8\t.\t+\t.\tID=Mp1g00020;Name=bad"
        )
        .unwrap();

        let assembly = AssemblyAccession::new("GCA_test").unwrap();
        let error = parse_gff3(&path, &assembly).expect_err("should fail");
        match error {
            StorageError::InvalidGffValue { line, .. } => assert_eq!(line, 2),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parse_gff3_reports_line_number_on_domain_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("invalid.gff");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "##gff-version 3").unwrap();
        writeln!(file, "chr1\tsrc\tgene\t1\t8\t.\t+\t.\tID=Mp1g00010;Name=ok").unwrap();
        // mRNA without a Parent attribute is parsed successfully by noodles
        // but rejected downstream — exercising the `line_number` value passed
        // to `parse_transcript`.
        writeln!(file, "chr1\tsrc\tmRNA\t1\t8\t.\t+\t.\tID=Mp1g00010.1").unwrap();

        let assembly = AssemblyAccession::new("GCA_test").unwrap();
        let error = parse_gff3(&path, &assembly).expect_err("should fail");
        match error {
            StorageError::MissingGffAttribute { line, attribute } => {
                assert_eq!(line, 2);
                assert_eq!(attribute, "Parent");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
