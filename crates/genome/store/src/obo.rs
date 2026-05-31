//! Thin wrapper around [`fastobo`] for loading GO term metadata from `.obo` files.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use fastobo::ast::{EntityFrame, OboDoc, TermClause, TermFrame};
use flate2::read::MultiGzDecoder;
use genome_domain::{GoNamespace, GoTermId};
use goterm_semsim::{GoDag, GoNode};

use crate::error::StorageError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoTerm {
    pub id: GoTermId,
    pub name: Option<String>,
    pub namespace: Option<GoNamespace>,
    pub is_a: Vec<GoTermId>,
    pub part_of: Vec<GoTermId>,
    pub alt_ids: Vec<GoTermId>,
    pub is_obsolete: bool,
}

#[derive(Debug, Clone, Default)]
pub struct GoOntology {
    terms: HashMap<GoTermId, GoTerm>,
    alt_to_primary: HashMap<GoTermId, GoTermId>,
}

impl GoOntology {
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn get(&self, id: &GoTermId) -> Option<&GoTerm> {
        self.terms.get(id)
    }

    /// Look up a term by its primary id or any of its `alt_id` aliases.
    pub fn resolve(&self, id: &GoTermId) -> Option<&GoTerm> {
        self.terms.get(id).or_else(|| {
            self.alt_to_primary
                .get(id)
                .and_then(|primary| self.terms.get(primary))
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &GoTerm> {
        self.terms.values()
    }

    /// Project this ontology into a [`GoDag`] for semantic-similarity
    /// computation. Obsolete terms are dropped (they don't contribute
    /// useful structure and break IC normalisation).
    pub fn to_dag(&self) -> GoDag {
        let mut builder = GoDag::builder();
        for term in self.terms.values() {
            if term.is_obsolete {
                continue;
            }
            builder.insert(GoNode {
                id: term.id.clone(),
                namespace: term.namespace,
                is_a: term.is_a.clone(),
                part_of: term.part_of.clone(),
            });
        }
        for (alt, primary) in &self.alt_to_primary {
            builder.alias(alt.clone(), primary.clone());
        }
        builder.build()
    }
}

pub fn load_go_ontology(path: impl AsRef<Path>) -> Result<GoOntology, StorageError> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let is_gz = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"));
    if is_gz {
        parse_obo_reader(BufReader::new(MultiGzDecoder::new(file)))
    } else {
        parse_obo_reader(BufReader::new(file))
    }
}

fn parse_obo_reader(reader: impl BufRead) -> Result<GoOntology, StorageError> {
    let doc =
        fastobo::from_reader(reader).map_err(|err| StorageError::InvalidObo(err.to_string()))?;
    Ok(build_go_ontology(&doc))
}

fn build_go_ontology(doc: &OboDoc) -> GoOntology {
    let mut ontology = GoOntology::default();
    for entity in doc.entities() {
        let EntityFrame::Term(term_frame) = entity else {
            continue;
        };
        let Some(term) = parse_term_frame(term_frame) else {
            continue;
        };
        for alt_id in &term.alt_ids {
            ontology
                .alt_to_primary
                .insert(alt_id.clone(), term.id.clone());
        }
        ontology.terms.insert(term.id.clone(), term);
    }
    ontology
}

fn parse_term_frame(term_frame: &TermFrame) -> Option<GoTerm> {
    let id = parse_go_term_id(&term_frame.id().as_inner().to_string())?;

    let mut go_term = GoTerm {
        id,
        name: None,
        namespace: None,
        is_a: Vec::new(),
        part_of: Vec::new(),
        alt_ids: Vec::new(),
        is_obsolete: false,
    };

    for clause in term_frame.iter() {
        apply_term_clause(&mut go_term, clause.as_inner());
    }

    Some(go_term)
}

fn apply_term_clause(go_term: &mut GoTerm, clause: &TermClause) {
    match clause {
        TermClause::Name(name) => {
            go_term.name = Some(name.as_str().to_owned());
        }
        TermClause::Namespace(namespace) => {
            go_term.namespace = parse_go_namespace(&namespace.to_string());
        }
        TermClause::IsA(parent) => push_go_id(&mut go_term.is_a, &parent.to_string()),
        TermClause::Relationship(rel, target) if rel.to_string() == "part_of" => {
            push_go_id(&mut go_term.part_of, &target.to_string());
        }
        TermClause::AltId(alt) => push_go_id(&mut go_term.alt_ids, &alt.to_string()),
        TermClause::IsObsolete(flag) => {
            go_term.is_obsolete = *flag;
        }
        _ => {}
    }
}

fn push_go_id(values: &mut Vec<GoTermId>, value: &str) {
    if let Some(id) = parse_go_term_id(value) {
        values.push(id);
    }
}

fn parse_go_term_id(value: &str) -> Option<GoTermId> {
    GoTermId::new(value).ok()
}

fn parse_go_namespace(value: &str) -> Option<GoNamespace> {
    match value {
        "biological_process" => Some(GoNamespace::BiologicalProcess),
        "molecular_function" => Some(GoNamespace::MolecularFunction),
        "cellular_component" => Some(GoNamespace::CellularComponent),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::GzEncoder;

    use super::*;

    const SAMPLE_OBO: &str = "format-version: 1.2\n\
        ontology: go\n\
        \n\
        [Term]\n\
        id: GO:0008150\n\
        name: biological_process\n\
        namespace: biological_process\n\
        alt_id: GO:0000004\n\
        alt_id: GO:0007582\n\
        def: \"A biological process.\" [GOC:test]\n\
        \n\
        [Term]\n\
        id: GO:0003674\n\
        name: molecular_function\n\
        namespace: molecular_function\n\
        \n\
        [Term]\n\
        id: GO:0009987\n\
        name: cellular process\n\
        namespace: biological_process\n\
        is_a: GO:0008150 ! biological_process\n\
        \n\
        [Term]\n\
        id: GO:0044238\n\
        name: primary metabolic process\n\
        namespace: biological_process\n\
        is_a: GO:0009987 ! cellular process\n\
        relationship: part_of GO:0008150 ! biological_process\n\
        \n\
        [Term]\n\
        id: GO:0000000\n\
        name: obsolete example\n\
        namespace: biological_process\n\
        is_obsolete: true\n\
        \n\
        [Typedef]\n\
        id: part_of\n\
        name: part of\n";

    #[test]
    fn loads_terms_with_name_namespace_and_parents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("go.obo");
        std::fs::write(&path, SAMPLE_OBO).unwrap();

        let ontology = load_go_ontology(&path).unwrap();
        assert_eq!(ontology.len(), 5);

        let bp = ontology
            .get(&GoTermId::new("GO:0008150").unwrap())
            .expect("biological_process term");
        assert_eq!(bp.name.as_deref(), Some("biological_process"));
        assert_eq!(bp.namespace, Some(GoNamespace::BiologicalProcess));
        assert!(bp.is_a.is_empty());
        assert!(bp.part_of.is_empty());

        let mf = ontology
            .get(&GoTermId::new("GO:0003674").unwrap())
            .expect("molecular_function term");
        assert_eq!(mf.namespace, Some(GoNamespace::MolecularFunction));

        let cellular = ontology
            .get(&GoTermId::new("GO:0009987").unwrap())
            .expect("cellular process term");
        assert_eq!(cellular.is_a, vec![GoTermId::new("GO:0008150").unwrap()]);
    }

    #[test]
    fn captures_part_of_relationships() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("go.obo");
        std::fs::write(&path, SAMPLE_OBO).unwrap();

        let ontology = load_go_ontology(&path).unwrap();
        let metabolic = ontology
            .get(&GoTermId::new("GO:0044238").unwrap())
            .expect("primary metabolic process term");
        assert_eq!(metabolic.is_a, vec![GoTermId::new("GO:0009987").unwrap()]);
        assert_eq!(
            metabolic.part_of,
            vec![GoTermId::new("GO:0008150").unwrap()]
        );
    }

    #[test]
    fn ignores_non_part_of_relationships() {
        let obo = "format-version: 1.2\n\
            ontology: go\n\
            \n\
            [Term]\n\
            id: GO:0008150\n\
            name: biological_process\n\
            namespace: biological_process\n\
            \n\
            [Term]\n\
            id: GO:0009987\n\
            name: cellular process\n\
            namespace: biological_process\n\
            relationship: regulates GO:0008150 ! biological_process\n";
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("go.obo");
        std::fs::write(&path, obo).unwrap();

        let ontology = load_go_ontology(&path).unwrap();
        let cellular = ontology
            .get(&GoTermId::new("GO:0009987").unwrap())
            .expect("cellular process term");
        assert!(cellular.part_of.is_empty());
    }

    #[test]
    fn is_empty_and_iter_reflect_loaded_terms() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("go.obo");
        std::fs::write(&path, SAMPLE_OBO).unwrap();
        let ontology = load_go_ontology(&path).unwrap();
        assert!(!ontology.is_empty());

        let ids: Vec<&str> = ontology.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids.len(), 5);
        assert!(ids.contains(&"GO:0008150"));

        let empty = GoOntology::default();
        assert!(empty.is_empty());
        assert_eq!(empty.iter().count(), 0);
    }

    #[test]
    fn records_obsolete_flag() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("go.obo");
        std::fs::write(&path, SAMPLE_OBO).unwrap();

        let ontology = load_go_ontology(&path).unwrap();
        let obsolete = ontology
            .get(&GoTermId::new("GO:0000000").unwrap())
            .expect("obsolete term");
        assert!(obsolete.is_obsolete);
    }

    #[test]
    fn resolves_alt_ids_to_primary_term() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("go.obo");
        std::fs::write(&path, SAMPLE_OBO).unwrap();

        let ontology = load_go_ontology(&path).unwrap();
        let primary = GoTermId::new("GO:0008150").unwrap();
        let alt = GoTermId::new("GO:0000004").unwrap();

        assert!(ontology.get(&alt).is_none());
        let resolved = ontology.resolve(&alt).expect("alt id resolves");
        assert_eq!(resolved.id, primary);
    }

    #[test]
    fn skips_typedef_frames() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("go.obo");
        std::fs::write(&path, SAMPLE_OBO).unwrap();

        let ontology = load_go_ontology(&path).unwrap();
        for term in ontology.iter() {
            assert!(term.id.as_str().starts_with("GO:"));
        }
    }

    #[test]
    fn reads_gzipped_input() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("go.obo.gz");
        let file = std::fs::File::create(&path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(SAMPLE_OBO.as_bytes()).unwrap();
        encoder.finish().unwrap();

        let ontology = load_go_ontology(&path).unwrap();
        assert_eq!(ontology.len(), 5);
        assert!(
            ontology
                .get(&GoTermId::new("GO:0008150").unwrap())
                .is_some()
        );
    }

    #[test]
    fn returns_invalid_obo_for_malformed_input() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.obo");
        std::fs::write(&path, "this is not a valid obo file\n[Term\nid: nope\n").unwrap();

        match load_go_ontology(&path) {
            Err(StorageError::InvalidObo(_)) => {}
            Err(other) => panic!("unexpected error: {other:?}"),
            Ok(_) => panic!("expected parse failure"),
        }
    }

    #[test]
    fn to_dag_drops_obsolete_terms_and_carries_relationships() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("go.obo");
        std::fs::write(&path, SAMPLE_OBO).unwrap();

        let ontology = load_go_ontology(&path).unwrap();
        let dag = ontology.to_dag();

        // 5 terms loaded, 1 obsolete dropped → 4 in the DAG.
        assert_eq!(dag.len(), 4);
        assert!(dag.get(&GoTermId::new("GO:0000000").unwrap()).is_none());

        let cellular = dag
            .get(&GoTermId::new("GO:0009987").unwrap())
            .expect("cellular process in DAG");
        assert_eq!(cellular.is_a, vec![GoTermId::new("GO:0008150").unwrap()]);

        // part_of edges survive the projection.
        let metabolic = dag
            .get(&GoTermId::new("GO:0044238").unwrap())
            .expect("primary metabolic process in DAG");
        assert_eq!(
            metabolic.part_of,
            vec![GoTermId::new("GO:0008150").unwrap()]
        );

        // alt_id resolution carries over.
        let primary = GoTermId::new("GO:0008150").unwrap();
        let alt = GoTermId::new("GO:0000004").unwrap();
        assert_eq!(dag.resolve(&alt), Some(&primary));
    }

    #[test]
    fn parse_go_namespace_recognises_three_branches() {
        assert_eq!(
            parse_go_namespace("biological_process"),
            Some(GoNamespace::BiologicalProcess)
        );
        assert_eq!(
            parse_go_namespace("molecular_function"),
            Some(GoNamespace::MolecularFunction)
        );
        assert_eq!(
            parse_go_namespace("cellular_component"),
            Some(GoNamespace::CellularComponent)
        );
        assert_eq!(parse_go_namespace("other_namespace"), None);
    }
}
