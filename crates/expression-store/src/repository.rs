use std::collections::HashMap;
use std::path::Path;

use expression_core::{
    BioProject, BioProjectAccession, BioSampleAccession, ExpressionMatrix, ExpressionMeasurement,
    ExpressionQuery, ExpressionRepository, ExpressionUnit, Sample, SraRunAccession,
};
use genome_core::{AssemblyAccession, GeneId};

use crate::dataset::ExpressionDataset;
use crate::error::ExpressionStoreError;
use crate::snapshot::read_snapshot;

/// In-memory [`ExpressionRepository`] backed by an [`ExpressionDataset`].
///
/// Construction is O(N) in samples + matrix cells: a few lookup tables are
/// built up front so per-query work stays bounded by what's actually needed
/// (the matrix row for a gene, or the requested gene × run sub-rectangle).
///
/// The repository assumes at most one [`ExpressionMatrix`] per
/// [`ExpressionUnit`] within a dataset; duplicates are rejected at construction
/// so callers can rely on `expression_matrix(.., unit)` having a single
/// canonical source.
#[derive(Debug, Clone)]
pub struct FileExpressionRepository {
    dataset: ExpressionDataset,
    sample_by_run: HashMap<SraRunAccession, Sample>,
    bioproject_by_accession: HashMap<BioProjectAccession, BioProject>,
    matrix_by_unit: HashMap<ExpressionUnit, MatrixIndex>,
}

#[derive(Debug, Clone)]
struct MatrixIndex {
    matrix: ExpressionMatrix,
    gene_index: HashMap<GeneId, usize>,
    run_index: HashMap<SraRunAccession, usize>,
}

impl MatrixIndex {
    fn new(matrix: ExpressionMatrix) -> Self {
        let gene_index = matrix
            .gene_ids
            .iter()
            .enumerate()
            .map(|(idx, gene_id)| (gene_id.clone(), idx))
            .collect();
        let run_index = matrix
            .runs
            .iter()
            .enumerate()
            .map(|(idx, run)| (run.clone(), idx))
            .collect();
        Self {
            matrix,
            gene_index,
            run_index,
        }
    }
}

impl FileExpressionRepository {
    pub fn new(dataset: ExpressionDataset) -> Result<Self, ExpressionStoreError> {
        let sample_by_run = dataset
            .samples
            .iter()
            .map(|sample| (sample.run().clone(), sample.clone()))
            .collect();
        let bioproject_by_accession = dataset
            .bioprojects
            .iter()
            .map(|project| (project.accession.clone(), project.clone()))
            .collect();

        let mut matrix_by_unit = HashMap::new();
        for matrix in &dataset.matrices {
            if matrix.assembly_accession != dataset.assembly_accession {
                return Err(ExpressionStoreError::MatrixAssemblyMismatch {
                    matrix: matrix.assembly_accession.to_string(),
                    dataset: dataset.assembly_accession.to_string(),
                });
            }
            if matrix_by_unit
                .insert(matrix.unit, MatrixIndex::new(matrix.clone()))
                .is_some()
            {
                return Err(ExpressionStoreError::DuplicateMatrix(matrix.unit));
            }
        }

        Ok(Self {
            dataset,
            sample_by_run,
            bioproject_by_accession,
            matrix_by_unit,
        })
    }

    pub fn from_snapshot_path(path: impl AsRef<Path>) -> Result<Self, ExpressionStoreError> {
        let snapshot = read_snapshot(path)?;
        Self::new(snapshot.dataset)
    }

    pub fn assembly_accession(&self) -> &AssemblyAccession {
        &self.dataset.assembly_accession
    }

    pub fn units(&self) -> Vec<ExpressionUnit> {
        let mut units: Vec<_> = self.matrix_by_unit.keys().copied().collect();
        units.sort();
        units
    }

    fn passes_run_filter(&self, run: &SraRunAccession, query: &ExpressionQuery) -> bool {
        query
            .runs
            .as_ref()
            .is_none_or(|allowed| allowed.contains(run))
    }

    /// Returns `true` if the sample for `run` matches the study / bioproject
    /// filters in `query`. Runs with no sample metadata pass when neither
    /// filter is set, and fail when either filter is set (we can't prove they
    /// match without metadata).
    fn passes_sample_filter(&self, run: &SraRunAccession, query: &ExpressionQuery) -> bool {
        if query.study.is_none() && query.bioproject.is_none() {
            return true;
        }
        let Some(sample) = self.sample_by_run.get(run) else {
            return false;
        };
        if let Some(study) = &query.study
            && sample.study() != Some(study)
        {
            return false;
        }
        if let Some(bioproject) = &query.bioproject
            && sample.bioproject() != Some(bioproject)
        {
            return false;
        }
        true
    }

    fn units_for_query(&self, query: &ExpressionQuery) -> Vec<ExpressionUnit> {
        let mut units: Vec<ExpressionUnit> = match query.unit {
            Some(unit) => vec![unit],
            None => self.matrix_by_unit.keys().copied().collect(),
        };
        // Stable order so callers (and tests) see deterministic output.
        units.sort();
        units
    }

    fn append_gene_expression_for_unit(
        &self,
        out: &mut Vec<ExpressionMeasurement>,
        gene_id: &GeneId,
        query: &ExpressionQuery,
        unit: ExpressionUnit,
        limit: usize,
    ) -> bool {
        let Some(index) = self.matrix_by_unit.get(&unit) else {
            return false;
        };
        let Some(&gene_idx) = index.gene_index.get(gene_id) else {
            return false;
        };
        let Some(row) = index.matrix.gene_row(gene_idx) else {
            return false;
        };

        for (run, &value) in index.matrix.runs.iter().zip(row.iter()) {
            if value.is_finite()
                && self.passes_run_filter(run, query)
                && self.passes_sample_filter(run, query)
            {
                out.push(ExpressionMeasurement {
                    gene_id: gene_id.clone(),
                    run: run.clone(),
                    value,
                    unit,
                });
                if out.len() >= limit {
                    return true;
                }
            }
        }

        false
    }
}

impl ExpressionRepository for FileExpressionRepository {
    fn sample(&self, run: &SraRunAccession) -> Option<Sample> {
        self.sample_by_run.get(run).cloned()
    }

    fn samples_for_assembly(&self, accession: &AssemblyAccession) -> Vec<Sample> {
        if &self.dataset.assembly_accession != accession {
            return Vec::new();
        }
        self.dataset.samples.clone()
    }

    fn samples_for_bioproject(&self, accession: &BioProjectAccession) -> Vec<Sample> {
        self.dataset
            .samples
            .iter()
            .filter(|sample| sample.bioproject() == Some(accession))
            .cloned()
            .collect()
    }

    fn samples_for_biosample(&self, accession: &BioSampleAccession) -> Vec<Sample> {
        self.dataset
            .samples
            .iter()
            .filter(|sample| sample.biosample() == Some(accession))
            .cloned()
            .collect()
    }

    fn bioproject(&self, accession: &BioProjectAccession) -> Option<BioProject> {
        self.bioproject_by_accession.get(accession).cloned()
    }

    fn gene_expression(
        &self,
        gene_id: &GeneId,
        query: &ExpressionQuery,
    ) -> Vec<ExpressionMeasurement> {
        let limit = query.limit.unwrap_or(usize::MAX);
        let mut out = Vec::new();

        for unit in self.units_for_query(query) {
            if self.append_gene_expression_for_unit(&mut out, gene_id, query, unit, limit) {
                break;
            }
        }

        out
    }

    fn expression_matrix(
        &self,
        accession: &AssemblyAccession,
        gene_ids: &[GeneId],
        runs: &[SraRunAccession],
        unit: ExpressionUnit,
    ) -> Option<ExpressionMatrix> {
        if &self.dataset.assembly_accession != accession {
            return None;
        }
        let index = self.matrix_by_unit.get(&unit)?;

        let gene_indices = gene_ids
            .iter()
            .map(|gene_id| index.gene_index.get(gene_id).copied())
            .collect::<Option<Vec<_>>>()?;
        let run_indices = runs
            .iter()
            .map(|run| index.run_index.get(run).copied())
            .collect::<Option<Vec<_>>>()?;

        let mut values = Vec::with_capacity(gene_indices.len().saturating_mul(run_indices.len()));
        for &gene_idx in &gene_indices {
            for &run_idx in &run_indices {
                values.push(index.matrix.value(gene_idx, run_idx)?);
            }
        }

        ExpressionMatrix::new(
            accession.clone(),
            unit,
            gene_ids.to_vec(),
            runs.to_vec(),
            values,
        )
        .ok()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::collections::BTreeMap;

    use expression_core::{
        BioProject, BioProjectAccession, BioSampleAccession, ExpressionMatrix, ExpressionUnit,
        Sample, SampleIdentity, SampleMetadata, SraExperimentAccession, SraRunAccession,
        SraStudyAccession,
    };
    use genome_core::{AssemblyAccession, GeneId};

    use super::*;

    const ASSEMBLY: &str = "GCA_037833805.1";

    fn assembly() -> AssemblyAccession {
        AssemblyAccession::new(ASSEMBLY).unwrap()
    }

    fn run(value: &str) -> SraRunAccession {
        SraRunAccession::new(value).unwrap()
    }

    fn gene(value: &str) -> GeneId {
        GeneId::new(value).unwrap()
    }

    fn make_sample(
        run_id: &str,
        study: Option<&str>,
        bioproject: Option<&str>,
        biosample: Option<&str>,
    ) -> Sample {
        Sample {
            identity: SampleIdentity {
                run: run(run_id),
                experiment: Some(SraExperimentAccession::new("SRX000001").unwrap()),
                study: study.map(|s| SraStudyAccession::new(s).unwrap()),
                biosample: biosample.map(|s| BioSampleAccession::new(s).unwrap()),
                bioproject: bioproject.map(|s| BioProjectAccession::new(s).unwrap()),
                assembly_accession: assembly(),
                title: None,
                description: None,
                library_strategy: None,
                library_layout: None,
                platform: None,
                instrument_model: None,
            },
            metadata: SampleMetadata::default(),
        }
    }

    /// Three samples × three runs over two studies / one bioproject, with
    /// matching TPM and raw-count matrices.
    fn make_dataset() -> ExpressionDataset {
        let samples = vec![
            make_sample(
                "SRR000001",
                Some("SRP000001"),
                Some("PRJNA1"),
                Some("SAMN1"),
            ),
            make_sample(
                "SRR000002",
                Some("SRP000001"),
                Some("PRJNA1"),
                Some("SAMN2"),
            ),
            make_sample(
                "SRR000003",
                Some("SRP000002"),
                Some("PRJNA2"),
                Some("SAMN3"),
            ),
        ];
        let bioprojects = vec![
            BioProject {
                accession: BioProjectAccession::new("PRJNA1").unwrap(),
                title: "Project One".to_owned(),
                description: None,
                attributes: BTreeMap::new(),
            },
            BioProject {
                accession: BioProjectAccession::new("PRJNA2").unwrap(),
                title: "Project Two".to_owned(),
                description: None,
                attributes: BTreeMap::new(),
            },
        ];
        let gene_ids = vec![gene("Mp1g00010"), gene("Mp1g00020")];
        let runs = vec![run("SRR000001"), run("SRR000002"), run("SRR000003")];

        let tpm = ExpressionMatrix::new(
            assembly(),
            ExpressionUnit::Tpm,
            gene_ids.clone(),
            runs.clone(),
            vec![
                10.0,
                20.0,
                30.0, // Mp1g00010
                40.0,
                50.0,
                f64::NAN, // Mp1g00020 (NaN = missing)
            ],
        )
        .unwrap();
        let raw = ExpressionMatrix::new(
            assembly(),
            ExpressionUnit::RawCount,
            gene_ids,
            runs,
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        )
        .unwrap();

        ExpressionDataset {
            assembly_accession: assembly(),
            bioprojects,
            samples,
            matrices: vec![tpm, raw],
        }
    }

    fn make_repository() -> FileExpressionRepository {
        FileExpressionRepository::new(make_dataset()).unwrap()
    }

    #[test]
    fn rejects_duplicate_unit_matrices() {
        let mut dataset = make_dataset();
        let dup = dataset.matrices[0].clone();
        dataset.matrices.push(dup);
        let err = FileExpressionRepository::new(dataset).unwrap_err();
        assert!(matches!(
            err,
            ExpressionStoreError::DuplicateMatrix(ExpressionUnit::Tpm)
        ));
    }

    #[test]
    fn rejects_matrix_from_other_assembly() {
        let mut dataset = make_dataset();
        let other = AssemblyAccession::new("GCA_other").unwrap();
        dataset.matrices[0].assembly_accession = other;
        let err = FileExpressionRepository::new(dataset).unwrap_err();
        assert!(matches!(
            err,
            ExpressionStoreError::MatrixAssemblyMismatch { .. }
        ));
    }

    #[test]
    fn sample_lookup_by_run() {
        let repo = make_repository();
        assert_eq!(
            repo.sample(&run("SRR000001")).unwrap().identity.run,
            run("SRR000001")
        );
        assert!(repo.sample(&run("SRR999999")).is_none());
    }

    #[test]
    fn samples_for_assembly_matches_only_dataset_assembly() {
        let repo = make_repository();
        assert_eq!(repo.samples_for_assembly(&assembly()).len(), 3);
        let other = AssemblyAccession::new("GCA_other").unwrap();
        assert!(repo.samples_for_assembly(&other).is_empty());
    }

    #[test]
    fn samples_for_bioproject_and_biosample_filter_correctly() {
        let repo = make_repository();
        let prj1 = BioProjectAccession::new("PRJNA1").unwrap();
        assert_eq!(repo.samples_for_bioproject(&prj1).len(), 2);

        let unknown_prj = BioProjectAccession::new("PRJNA9999").unwrap();
        assert!(repo.samples_for_bioproject(&unknown_prj).is_empty());

        let bs2 = BioSampleAccession::new("SAMN2").unwrap();
        assert_eq!(repo.samples_for_biosample(&bs2).len(), 1);
    }

    #[test]
    fn bioproject_lookup() {
        let repo = make_repository();
        let prj1 = BioProjectAccession::new("PRJNA1").unwrap();
        assert_eq!(repo.bioproject(&prj1).unwrap().title, "Project One");

        let unknown = BioProjectAccession::new("PRJNA9999").unwrap();
        assert!(repo.bioproject(&unknown).is_none());
    }

    #[test]
    fn gene_expression_with_no_filters_returns_all_units_skipping_nan() {
        let repo = make_repository();
        let result = repo.gene_expression(&gene("Mp1g00020"), &ExpressionQuery::default());
        // Gene Mp1g00020 has 1 NaN in TPM (SRR000003) + 3 raw counts = 5 measurements.
        assert_eq!(result.len(), 5);
        assert!(
            result
                .iter()
                .all(|measurement| measurement.gene_id == gene("Mp1g00020"))
        );
    }

    #[test]
    fn gene_expression_filters_by_unit() {
        let repo = make_repository();
        let query = ExpressionQuery {
            unit: Some(ExpressionUnit::Tpm),
            ..Default::default()
        };
        let result = repo.gene_expression(&gene("Mp1g00010"), &query);
        assert_eq!(result.len(), 3);
        assert!(
            result
                .iter()
                .all(|measurement| measurement.unit == ExpressionUnit::Tpm)
        );
    }

    #[test]
    fn gene_expression_filters_by_runs() {
        let repo = make_repository();
        let query = ExpressionQuery {
            runs: Some(vec![run("SRR000001"), run("SRR000002")]),
            unit: Some(ExpressionUnit::Tpm),
            ..Default::default()
        };
        let result = repo.gene_expression(&gene("Mp1g00010"), &query);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].run, run("SRR000001"));
        assert_eq!(result[1].run, run("SRR000002"));
    }

    #[test]
    fn gene_expression_filters_by_study_and_bioproject() {
        let repo = make_repository();
        let query = ExpressionQuery {
            study: Some(SraStudyAccession::new("SRP000001").unwrap()),
            unit: Some(ExpressionUnit::Tpm),
            ..Default::default()
        };
        let result = repo.gene_expression(&gene("Mp1g00010"), &query);
        assert_eq!(result.len(), 2);

        let query = ExpressionQuery {
            bioproject: Some(BioProjectAccession::new("PRJNA2").unwrap()),
            unit: Some(ExpressionUnit::Tpm),
            ..Default::default()
        };
        let result = repo.gene_expression(&gene("Mp1g00010"), &query);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].run, run("SRR000003"));
    }

    #[test]
    fn gene_expression_returns_empty_for_unknown_gene() {
        let repo = make_repository();
        let result = repo.gene_expression(&gene("Mp9g99999"), &ExpressionQuery::default());
        assert!(result.is_empty());
    }

    #[test]
    fn gene_expression_respects_limit() {
        let repo = make_repository();
        let query = ExpressionQuery {
            limit: Some(2),
            ..Default::default()
        };
        let result = repo.gene_expression(&gene("Mp1g00010"), &query);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn expression_matrix_subset_lookup() {
        let repo = make_repository();
        let result = repo
            .expression_matrix(
                &assembly(),
                &[gene("Mp1g00020"), gene("Mp1g00010")],
                &[run("SRR000002"), run("SRR000001")],
                ExpressionUnit::Tpm,
            )
            .unwrap();
        assert_eq!(result.gene_count(), 2);
        assert_eq!(result.run_count(), 2);
        assert_eq!(result.value(0, 0), Some(50.0)); // Mp1g00020 × SRR000002
        assert_eq!(result.value(0, 1), Some(40.0)); // Mp1g00020 × SRR000001
        assert_eq!(result.value(1, 0), Some(20.0)); // Mp1g00010 × SRR000002
        assert_eq!(result.value(1, 1), Some(10.0)); // Mp1g00010 × SRR000001
    }

    #[test]
    fn expression_matrix_returns_none_for_unknown_gene_or_run() {
        let repo = make_repository();
        assert!(
            repo.expression_matrix(
                &assembly(),
                &[gene("Mp9g99999")],
                &[run("SRR000001")],
                ExpressionUnit::Tpm,
            )
            .is_none()
        );
        assert!(
            repo.expression_matrix(
                &assembly(),
                &[gene("Mp1g00010")],
                &[run("SRR999999")],
                ExpressionUnit::Tpm,
            )
            .is_none()
        );
    }

    #[test]
    fn expression_matrix_returns_none_for_unknown_unit_or_assembly() {
        let repo = make_repository();
        assert!(
            repo.expression_matrix(
                &assembly(),
                &[gene("Mp1g00010")],
                &[run("SRR000001")],
                ExpressionUnit::Fpkm,
            )
            .is_none()
        );
        let other = AssemblyAccession::new("GCA_other").unwrap();
        assert!(
            repo.expression_matrix(
                &other,
                &[gene("Mp1g00010")],
                &[run("SRR000001")],
                ExpressionUnit::Tpm,
            )
            .is_none()
        );
    }

    #[test]
    fn units_listed_sorted() {
        let repo = make_repository();
        let mut expected = vec![ExpressionUnit::Tpm, ExpressionUnit::RawCount];
        expected.sort();
        assert_eq!(repo.units(), expected);
    }
}
