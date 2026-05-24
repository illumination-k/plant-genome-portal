use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use genome_core::GeneId;

use crate::error::StorageError;
use crate::gff::ParsedGff;
use crate::util::clean_optional_value;

#[derive(Debug, Clone, Default)]
pub(crate) struct NomenclatureEntry {
    pub attributes: BTreeMap<String, String>,
}

pub(crate) fn apply_nomenclature(
    parsed: &mut ParsedGff,
    nomenclature: &HashMap<GeneId, NomenclatureEntry>,
) {
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

pub(crate) fn parse_nomenclature(
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
