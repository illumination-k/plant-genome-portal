use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use genome_core::{
    AnnotationEvidence, AnnotationSource, FunctionalAnnotation, GeneId, GoTermAnnotation, GoTermId,
    InterProAnnotation, InterProId, KeggAnnotation, KeggEntryId, KogAnnotation, KogEntryId,
    NcbiFamAccession, NcbiFamAnnotation, PfamAccession, PfamAnnotation, TranscriptId,
};

use crate::error::StorageError;
use crate::gff::ParsedGff;
use crate::util::clean_optional_value;

pub(crate) fn apply_functional_annotations(
    parsed: &mut ParsedGff,
    annotations: &HashMap<TranscriptId, Vec<FunctionalAnnotation>>,
) {
    let mut annotations_by_gene: HashMap<GeneId, Vec<FunctionalAnnotation>> = HashMap::new();

    for transcript in &mut parsed.transcripts {
        let Some(transcript_annotations) = annotations.get(&transcript.id) else {
            continue;
        };
        transcript.annotations = transcript_annotations.clone();
        annotations_by_gene
            .entry(transcript.gene_id.clone())
            .or_default()
            .extend(transcript_annotations.iter().cloned());
    }

    for gene in &mut parsed.genes {
        let Some(gene_annotations) = annotations_by_gene.get(&gene.id) else {
            continue;
        };
        gene.annotations = unique_annotations(gene_annotations.iter().cloned());
    }
}

pub(crate) fn parse_functional_annotations(
    path: impl AsRef<Path>,
) -> Result<HashMap<TranscriptId, Vec<FunctionalAnnotation>>, StorageError> {
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
        annotations.insert(
            TranscriptId::new(transcript_id)?,
            parse_functional_annotation_value(annotation),
        );
    }

    Ok(annotations)
}

fn parse_functional_annotation_value(value: &str) -> Vec<FunctionalAnnotation> {
    unique_annotations(
        value
            .split(';')
            .filter_map(parse_functional_annotation_part),
    )
}

fn parse_functional_annotation_part(value: &str) -> Option<FunctionalAnnotation> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    parse_kegg_annotation(value)
        .or_else(|| parse_go_annotation(value))
        .or_else(|| parse_interpro_annotation(value))
        .or_else(|| parse_pfam_annotation(value))
        .or_else(|| parse_ncbi_fam_annotation(value))
        .or_else(|| parse_kog_annotation(value))
}

fn parse_kegg_annotation(value: &str) -> Option<FunctionalAnnotation> {
    let rest = value.strip_prefix("KEGG:")?;
    let (entry_id, name) = split_kegg_id_and_name(rest);
    let entry_id = KeggEntryId::new(entry_id).ok()?;
    Some(FunctionalAnnotation::Kegg(KeggAnnotation::new(
        entry_id,
        clean_optional_value(name),
        AnnotationEvidence::new(AnnotationSource::Kegg),
    )))
}

fn parse_go_annotation(value: &str) -> Option<FunctionalAnnotation> {
    let go_id = value.get(..10)?;
    let term_id = GoTermId::new(go_id).ok()?;
    let name = value
        .get(10..)
        .and_then(|rest| rest.strip_prefix(':'))
        .and_then(clean_optional_value);
    Some(FunctionalAnnotation::GoTerm(GoTermAnnotation {
        term_id,
        name,
        namespace: None,
        evidence: AnnotationEvidence::new(AnnotationSource::Go),
    }))
}

fn parse_interpro_annotation(value: &str) -> Option<FunctionalAnnotation> {
    let rest = value.strip_prefix("InterPro:").unwrap_or(value);
    let (interpro_id, name) = split_annotation_id_and_name(rest);
    let interpro_id = InterProId::new(interpro_id).ok()?;
    Some(FunctionalAnnotation::InterPro(InterProAnnotation {
        interpro_id,
        name: clean_optional_value(name),
        evidence: AnnotationEvidence::new(AnnotationSource::InterProScan),
    }))
}

fn parse_pfam_annotation(value: &str) -> Option<FunctionalAnnotation> {
    let rest = value.strip_prefix("Pfam:")?;
    let (accession, name) = split_annotation_id_and_name(rest);
    Some(FunctionalAnnotation::Pfam(PfamAnnotation {
        accession: PfamAccession::new(accession).ok()?,
        name: clean_optional_value(name),
        interpro_id: None,
        evidence: AnnotationEvidence::new(AnnotationSource::InterProScan),
    }))
}

fn parse_ncbi_fam_annotation(value: &str) -> Option<FunctionalAnnotation> {
    let rest = value.strip_prefix("NCBIfam:")?;
    let (accession, name) = split_annotation_id_and_name(rest);
    Some(FunctionalAnnotation::NcbiFam(NcbiFamAnnotation {
        accession: NcbiFamAccession::new(accession).ok()?,
        name: clean_optional_value(name),
        interpro_id: None,
        evidence: AnnotationEvidence::new(AnnotationSource::InterProScan),
    }))
}

fn parse_kog_annotation(value: &str) -> Option<FunctionalAnnotation> {
    let rest = value.strip_prefix("KOG:")?;
    let (entry_id, name) = split_annotation_id_and_name(rest);
    Some(FunctionalAnnotation::Kog(KogAnnotation {
        entry_id: KogEntryId::new(entry_id).ok()?,
        name: clean_optional_value(name),
        interpro_id: None,
        evidence: AnnotationEvidence::new(AnnotationSource::InterProScan),
    }))
}

fn split_annotation_id_and_name(value: &str) -> (&str, &str) {
    value.split_once(':').unwrap_or((value, ""))
}

fn split_kegg_id_and_name(value: &str) -> (&str, &str) {
    for prefix in ["ko:", "path:"] {
        if let Some(rest) = value.strip_prefix(prefix) {
            let (id, name) = split_annotation_id_and_name(rest);
            let id_end = prefix.len() + id.len();
            return (&value[..id_end], name);
        }
    }

    split_annotation_id_and_name(value)
}

fn unique_annotations(
    values: impl IntoIterator<Item = FunctionalAnnotation>,
) -> Vec<FunctionalAnnotation> {
    let mut annotations = Vec::new();
    for annotation in values {
        if !annotations.contains(&annotation) {
            annotations.push(annotation);
        }
    }
    annotations
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn split_kegg_includes_ko_prefix_in_entry_id() {
        let (id, name) = split_kegg_id_and_name("ko:K00001:example annotation");
        assert_eq!(id, "ko:K00001");
        assert_eq!(name, "example annotation");
    }

    #[test]
    fn split_kegg_includes_path_prefix_in_entry_id() {
        let (id, name) = split_kegg_id_and_name("path:map00010:Glycolysis");
        assert_eq!(id, "path:map00010");
        assert_eq!(name, "Glycolysis");
    }

    #[test]
    fn split_kegg_without_known_prefix_falls_back_to_plain_split() {
        let (id, name) = split_kegg_id_and_name("K00001:example annotation");
        assert_eq!(id, "K00001");
        assert_eq!(name, "example annotation");
    }

    #[test]
    fn parse_functional_annotations_reports_correct_line_number() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("func.tsv");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "Mp1g00010.1\tKEGG:K00001:ok").unwrap();
        writeln!(file, "missing-tab-column-on-line-two").unwrap();

        let error = parse_functional_annotations(&path).expect_err("should fail");
        match error {
            StorageError::InvalidTsvValue { line, .. } => assert_eq!(line, 2),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
