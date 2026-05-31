use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use genome_core::{
    AssemblyAccession, GeneId, Orthogroup, OrthogroupCatalog, OrthogroupId, OrthogroupMember, TaxId,
};

use crate::error::StorageError;
use crate::util::clean_optional_value;

const REQUIRED_COLUMNS: [&str; 4] = ["orthogroup_id", "gene_id", "tax_id", "scientific_name"];
const OPTIONAL_COLUMNS: [&str; 2] = ["assembly_accession", "symbol"];

pub(crate) fn parse_orthogroups(path: impl AsRef<Path>) -> Result<OrthogroupCatalog, StorageError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let Some(header) = lines.next().transpose()? else {
        return Ok(OrthogroupCatalog::default());
    };
    let header = OrthogroupHeader::parse(&header)?;
    let mut rows: BTreeMap<OrthogroupId, Vec<OrthogroupMember>> = BTreeMap::new();
    let mut seen = HashSet::new();

    for (index, line) in lines.enumerate() {
        let line_number = index + 2;
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != header.column_count {
            return Err(StorageError::InvalidTsvValue {
                line: line_number,
                message: format!(
                    "expected {} columns, got {}",
                    header.column_count,
                    columns.len()
                ),
            });
        }
        let (orthogroup_id, member) = parse_member_row(&columns, &header, line_number)?;
        if seen.insert((orthogroup_id.clone(), member.clone())) {
            rows.entry(orthogroup_id).or_default().push(member);
        }
    }

    Ok(OrthogroupCatalog {
        groups: rows
            .into_iter()
            .map(|(id, mut members)| {
                members.sort();
                Orthogroup { id, members }
            })
            .collect(),
    })
}

struct OrthogroupHeader {
    column_count: usize,
    assembly_accession: Option<usize>,
    symbol: Option<usize>,
}

impl OrthogroupHeader {
    fn parse(header: &str) -> Result<Self, StorageError> {
        let columns = header.split('\t').collect::<Vec<_>>();
        if columns.len() < REQUIRED_COLUMNS.len()
            || columns.len() > REQUIRED_COLUMNS.len() + OPTIONAL_COLUMNS.len()
        {
            return Err(StorageError::InvalidTsvValue {
                line: 1,
                message: "unexpected orthogroup header".to_owned(),
            });
        }
        for (index, expected) in REQUIRED_COLUMNS.iter().enumerate() {
            if columns[index] != *expected {
                return Err(StorageError::InvalidTsvValue {
                    line: 1,
                    message: format!("expected column {} to be {expected}", index + 1),
                });
            }
        }
        for (index, column) in columns.iter().skip(REQUIRED_COLUMNS.len()).enumerate() {
            let expected = OPTIONAL_COLUMNS[index];
            if *column != expected {
                return Err(StorageError::InvalidTsvValue {
                    line: 1,
                    message: format!("expected optional column {} to be {expected}", index + 1),
                });
            }
        }

        Ok(Self {
            column_count: columns.len(),
            assembly_accession: (columns.len() > 4).then_some(4),
            symbol: (columns.len() > 5).then_some(5),
        })
    }
}

fn parse_member_row(
    columns: &[&str],
    header: &OrthogroupHeader,
    line: usize,
) -> Result<(OrthogroupId, OrthogroupMember), StorageError> {
    let tax_id = columns[2].parse::<u32>().map(TaxId::new).map_err(|error| {
        StorageError::InvalidTsvValue {
            line,
            message: format!("invalid tax_id: {error}"),
        }
    })?;
    let scientific_name =
        clean_optional_value(columns[3]).ok_or_else(|| StorageError::InvalidTsvValue {
            line,
            message: "scientific_name is required".to_owned(),
        })?;
    let assembly_accession = header
        .assembly_accession
        .and_then(|index| clean_optional_value(columns[index]))
        .map(AssemblyAccession::new)
        .transpose()?;
    let symbol = header
        .symbol
        .and_then(|index| clean_optional_value(columns[index]));

    Ok((
        OrthogroupId::new(columns[0])?,
        OrthogroupMember {
            gene_id: GeneId::new(columns[1])?,
            tax_id,
            scientific_name: scientific_name.to_owned(),
            assembly_accession,
            symbol,
        },
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::io::Write;

    use super::*;

    fn write_fixture(body: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        write!(file, "{body}").unwrap();
        file
    }

    #[test]
    fn parse_orthogroups_accepts_full_schema_and_sorts() {
        let file = write_fixture(
            "orthogroup_id\tgene_id\ttax_id\tscientific_name\tassembly_accession\tsymbol\n\
             OG2\tMp2g00020\t3197\tMarchantia polymorpha\tGCA_test\tBAR\n\
             OG1\tMp1g00010\t3197\tMarchantia polymorpha\tGCA_test\tFOO\n",
        );

        let catalog = parse_orthogroups(file.path()).unwrap();

        assert_eq!(catalog.groups[0].id.as_str(), "OG1");
        assert_eq!(catalog.groups[0].members[0].symbol.as_deref(), Some("FOO"));
        assert_eq!(catalog.groups[1].id.as_str(), "OG2");
    }

    #[test]
    fn parse_orthogroups_accepts_missing_optional_columns() {
        let file = write_fixture(
            "orthogroup_id\tgene_id\ttax_id\tscientific_name\n\
             OG1\tAT1G01010\t3702\tArabidopsis thaliana\n",
        );

        let catalog = parse_orthogroups(file.path()).unwrap();

        assert_eq!(catalog.groups.len(), 1);
        assert!(catalog.groups[0].members[0].assembly_accession.is_none());
        assert!(catalog.groups[0].members[0].symbol.is_none());
    }

    #[test]
    fn parse_orthogroups_accepts_empty_optional_values() {
        let file = write_fixture(
            "orthogroup_id\tgene_id\ttax_id\tscientific_name\tassembly_accession\tsymbol\n\
             OG1\tAT1G01010\t3702\tArabidopsis thaliana\t\t\n",
        );

        let catalog = parse_orthogroups(file.path()).unwrap();

        assert!(catalog.groups[0].members[0].assembly_accession.is_none());
        assert!(catalog.groups[0].members[0].symbol.is_none());
    }

    #[test]
    fn parse_orthogroups_deduplicates_duplicate_rows() {
        let file = write_fixture(
            "orthogroup_id\tgene_id\ttax_id\tscientific_name\n\
             OG1\tAT1G01010\t3702\tArabidopsis thaliana\n\
             OG1\tAT1G01010\t3702\tArabidopsis thaliana\n",
        );

        let catalog = parse_orthogroups(file.path()).unwrap();

        assert_eq!(catalog.groups[0].members.len(), 1);
    }

    #[test]
    fn parse_orthogroups_rejects_invalid_tax_id() {
        let file = write_fixture(
            "orthogroup_id\tgene_id\ttax_id\tscientific_name\n\
             OG1\tAT1G01010\tnot-a-taxid\tArabidopsis thaliana\n",
        );

        let error = parse_orthogroups(file.path()).expect_err("should reject invalid tax id");

        match error {
            StorageError::InvalidTsvValue { line, .. } => assert_eq!(line, 2),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parse_orthogroups_rejects_missing_required_header() {
        let file = write_fixture("orthogroup_id\tgene_id\ttax_id\n");

        let error = parse_orthogroups(file.path()).expect_err("should reject missing header");

        match error {
            StorageError::InvalidTsvValue { line, .. } => assert_eq!(line, 1),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
