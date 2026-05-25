use std::sync::Arc;

use expression_core::{
    BioProject, BioProjectAccession, BioSampleAccession, ExpressionMatrix, ExpressionMeasurement,
    ExpressionQuery, ExpressionRepository, ExpressionUnit, Sample, SraRunAccession,
    SraStudyAccession,
};
use genome_core::{AssemblyAccession, GeneId};

use crate::ServiceError;

/// Validated query parameters for [`ExpressionService::gene_expression`].
///
/// Mirrors [`expression_core::ExpressionQuery`] but accepts raw string inputs
/// so the API layer can stay free of expression-core types.
#[derive(Debug, Default, Clone)]
pub struct GeneExpressionRequest {
    pub runs: Option<Vec<String>>,
    pub study: Option<String>,
    pub bioproject: Option<String>,
    pub unit: Option<ExpressionUnit>,
    pub limit: Option<usize>,
}

/// Validated request body for [`ExpressionService::expression_matrix`].
#[derive(Debug, Clone)]
pub struct ExpressionMatrixRequest {
    pub gene_ids: Vec<String>,
    pub runs: Vec<String>,
    pub unit: ExpressionUnit,
}

#[derive(Clone)]
pub struct ExpressionService<R> {
    repository: Arc<R>,
}

impl<R> ExpressionService<R>
where
    R: ExpressionRepository,
{
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    pub fn sample(&self, run: &str) -> Result<Sample, ServiceError> {
        let run = parse_run(run)?;
        self.repository
            .sample(&run)
            .ok_or_else(|| ServiceError::SampleNotFound(run.into_string()))
    }

    pub fn samples_for_assembly(&self, accession: &str) -> Result<Vec<Sample>, ServiceError> {
        let accession = parse_accession(accession)?;
        Ok(self.repository.samples_for_assembly(&accession))
    }

    pub fn samples_for_bioproject(&self, accession: &str) -> Result<Vec<Sample>, ServiceError> {
        let accession = parse_bioproject(accession)?;
        Ok(self.repository.samples_for_bioproject(&accession))
    }

    pub fn samples_for_biosample(&self, accession: &str) -> Result<Vec<Sample>, ServiceError> {
        let accession = parse_biosample(accession)?;
        Ok(self.repository.samples_for_biosample(&accession))
    }

    pub fn bioproject(&self, accession: &str) -> Result<BioProject, ServiceError> {
        let accession = parse_bioproject(accession)?;
        self.repository
            .bioproject(&accession)
            .ok_or_else(|| ServiceError::BioProjectNotFound(accession.into_string()))
    }

    pub fn gene_expression(
        &self,
        gene_id: &str,
        request: GeneExpressionRequest,
    ) -> Result<Vec<ExpressionMeasurement>, ServiceError> {
        let gene_id = parse_gene(gene_id)?;
        let query = build_expression_query(request)?;
        Ok(self.repository.gene_expression(&gene_id, &query))
    }

    pub fn expression_matrix(
        &self,
        accession: &str,
        request: ExpressionMatrixRequest,
    ) -> Result<ExpressionMatrix, ServiceError> {
        let accession = parse_accession(accession)?;
        let gene_ids = request
            .gene_ids
            .iter()
            .map(|id| parse_gene(id))
            .collect::<Result<Vec<_>, _>>()?;
        let runs = request
            .runs
            .iter()
            .map(|run| parse_run(run))
            .collect::<Result<Vec<_>, _>>()?;

        self.repository
            .expression_matrix(&accession, &gene_ids, &runs, request.unit)
            .ok_or_else(|| {
                ServiceError::InvalidRequest(
                    "no expression matrix available for the requested assembly, genes, runs, and unit"
                        .to_owned(),
                )
            })
    }
}

fn build_expression_query(request: GeneExpressionRequest) -> Result<ExpressionQuery, ServiceError> {
    let runs = request
        .runs
        .map(|values| values.iter().map(|value| parse_run(value)).collect())
        .transpose()?;
    let study = request.study.as_deref().map(parse_study).transpose()?;
    let bioproject = request
        .bioproject
        .as_deref()
        .map(parse_bioproject)
        .transpose()?;
    Ok(ExpressionQuery {
        runs,
        study,
        bioproject,
        unit: request.unit,
        limit: request.limit,
    })
}

fn parse_accession(value: &str) -> Result<AssemblyAccession, ServiceError> {
    AssemblyAccession::new(value).map_err(|error| ServiceError::InvalidRequest(error.to_string()))
}

fn parse_gene(value: &str) -> Result<GeneId, ServiceError> {
    GeneId::new(value).map_err(|error| ServiceError::InvalidRequest(error.to_string()))
}

fn parse_run(value: &str) -> Result<SraRunAccession, ServiceError> {
    SraRunAccession::new(value).map_err(|error| ServiceError::InvalidRequest(error.to_string()))
}

fn parse_study(value: &str) -> Result<SraStudyAccession, ServiceError> {
    SraStudyAccession::new(value).map_err(|error| ServiceError::InvalidRequest(error.to_string()))
}

fn parse_bioproject(value: &str) -> Result<BioProjectAccession, ServiceError> {
    BioProjectAccession::new(value).map_err(|error| ServiceError::InvalidRequest(error.to_string()))
}

fn parse_biosample(value: &str) -> Result<BioSampleAccession, ServiceError> {
    BioSampleAccession::new(value).map_err(|error| ServiceError::InvalidRequest(error.to_string()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::collections::BTreeMap;

    use expression_core::{
        BioProject, BioProjectAccession, BioSampleAccession, ExpressionMatrix, ExpressionUnit,
        Sample, SraExperimentAccession, SraStudyAccession,
    };
    use expression_store::{ExpressionDataset, FileExpressionRepository};
    use genome_core::AssemblyAccession;

    use super::*;

    const ASSEMBLY: &str = "GCA_037833805.1";

    fn assembly() -> AssemblyAccession {
        AssemblyAccession::new(ASSEMBLY).unwrap()
    }

    fn sample(run_id: &str) -> Sample {
        Sample {
            run: SraRunAccession::new(run_id).unwrap(),
            experiment: Some(SraExperimentAccession::new("SRX000001").unwrap()),
            study: Some(SraStudyAccession::new("SRP000001").unwrap()),
            biosample: Some(BioSampleAccession::new("SAMN1").unwrap()),
            bioproject: Some(BioProjectAccession::new("PRJNA1").unwrap()),
            assembly_accession: assembly(),
            title: None,
            tissue: None,
            developmental_stage: None,
            treatment: None,
            condition: None,
            replicate: None,
            library_strategy: None,
            library_layout: None,
            platform: None,
            instrument_model: None,
            description: None,
            attributes: BTreeMap::new(),
        }
    }

    fn make_service() -> ExpressionService<FileExpressionRepository> {
        let gene_ids = vec![GeneId::new("Mp1g00010").unwrap()];
        let runs = vec![
            SraRunAccession::new("SRR000001").unwrap(),
            SraRunAccession::new("SRR000002").unwrap(),
        ];
        let matrix = ExpressionMatrix::new(
            assembly(),
            ExpressionUnit::Tpm,
            gene_ids,
            runs,
            vec![1.0, 2.0],
        )
        .unwrap();
        let dataset = ExpressionDataset {
            assembly_accession: assembly(),
            bioprojects: vec![BioProject {
                accession: BioProjectAccession::new("PRJNA1").unwrap(),
                title: "Project One".to_owned(),
                description: None,
                attributes: BTreeMap::new(),
            }],
            samples: vec![sample("SRR000001"), sample("SRR000002")],
            matrices: vec![matrix],
        };
        ExpressionService::new(FileExpressionRepository::new(dataset).unwrap())
    }

    #[test]
    fn sample_not_found_maps_to_service_error() {
        let service = make_service();
        let err = service.sample("SRR999999").unwrap_err();
        assert!(matches!(err, ServiceError::SampleNotFound(_)));
    }

    #[test]
    fn invalid_run_accession_returns_invalid_request() {
        let service = make_service();
        let err = service.sample("not-an-accession").unwrap_err();
        assert!(matches!(err, ServiceError::InvalidRequest(_)));
    }

    #[test]
    fn bioproject_lookup() {
        let service = make_service();
        assert_eq!(
            service.bioproject("PRJNA1").unwrap().title,
            "Project One".to_owned()
        );
        assert!(matches!(
            service.bioproject("PRJNA9999").unwrap_err(),
            ServiceError::BioProjectNotFound(_)
        ));
    }

    #[test]
    fn gene_expression_filters_by_unit() {
        let service = make_service();
        let request = GeneExpressionRequest {
            unit: Some(ExpressionUnit::Tpm),
            ..GeneExpressionRequest::default()
        };
        let result = service.gene_expression("Mp1g00010", request).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn matrix_subset_lookup() {
        let service = make_service();
        let request = ExpressionMatrixRequest {
            gene_ids: vec!["Mp1g00010".to_owned()],
            runs: vec!["SRR000001".to_owned()],
            unit: ExpressionUnit::Tpm,
        };
        let matrix = service.expression_matrix(ASSEMBLY, request).unwrap();
        assert_eq!(matrix.value(0, 0), Some(1.0));
    }

    #[test]
    fn matrix_missing_gene_returns_invalid_request() {
        let service = make_service();
        let request = ExpressionMatrixRequest {
            gene_ids: vec!["Mp9g99999".to_owned()],
            runs: vec!["SRR000001".to_owned()],
            unit: ExpressionUnit::Tpm,
        };
        assert!(matches!(
            service.expression_matrix(ASSEMBLY, request).unwrap_err(),
            ServiceError::InvalidRequest(_)
        ));
    }
}
