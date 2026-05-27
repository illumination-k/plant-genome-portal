use axum::{
    Json,
    extract::{Path, Query, State},
};
use co_expression::{
    ClusterDendrogram, cluster_dendrogram_columns, cluster_dendrogram_rows, row_z_scores,
};
use expression_core::{
    ExpressionMatrix, ExpressionQuery, ExpressionRepository, ExpressionUnit, SraRunAccession,
};
use genome_core::{AssemblyAccession, GeneId};
use serde::{Deserialize, Serialize};
use service::ServiceError;
use utoipa::{IntoParams, ToSchema};

use crate::{ApiError, AppExpressionRepository, AppState};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub(crate) struct GeneExpressionQuery {
    unit: Option<ExpressionUnit>,
    limit: Option<usize>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeneExpressionPoint {
    gene_id: String,
    run: String,
    label: String,
    primary_group: Option<String>,
    value: f64,
    unit: ExpressionUnit,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExpressionClustergramQuery {
    assembly_accession: String,
    gene_ids: String,
    unit: Option<ExpressionUnit>,
    runs: Option<String>,
    limit: Option<usize>,
    drop_missing_genes: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExpressionClustergramResponse {
    assembly_accession: String,
    unit: ExpressionUnit,
    genes: Vec<ExpressionGeneLabel>,
    samples: Vec<ExpressionSampleLabel>,
    values: Vec<f64>,
    row_order: Vec<usize>,
    column_order: Vec<usize>,
    row_dendrogram: ClusterDendrogram,
    column_dendrogram: ClusterDendrogram,
    z_scores: Vec<f64>,
}

impl ExpressionClustergramResponse {
    fn empty(assembly_accession: String, unit: ExpressionUnit) -> Self {
        Self {
            assembly_accession,
            unit,
            genes: Vec::new(),
            samples: Vec::new(),
            values: Vec::new(),
            row_order: Vec::new(),
            column_order: Vec::new(),
            row_dendrogram: ClusterDendrogram {
                root: None,
                nodes: Vec::new(),
            },
            column_dendrogram: ClusterDendrogram {
                root: None,
                nodes: Vec::new(),
            },
            z_scores: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExpressionGeneLabel {
    gene_id: String,
    label: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExpressionSampleLabel {
    run: String,
    label: String,
    primary_group: Option<String>,
}

#[utoipa::path(
    get,
    path = "/v2/gene/id/{gene_id}/expression",
    params(
        ("gene_id" = String, Path, description = "Gene identifier"),
        GeneExpressionQuery,
    ),
    responses(
        (status = 200, description = "Expression values for one gene", body = Vec<GeneExpressionPoint>),
        (status = 404, description = "Gene not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn gene_expression(
    State(state): State<AppState>,
    Path(gene_id): Path<String>,
    Query(query): Query<GeneExpressionQuery>,
) -> Result<Json<Vec<GeneExpressionPoint>>, ApiError> {
    let gene_record = state.service.gene(&gene_id)?;
    let Some(expression_repository) = state.expression_repository.as_ref() else {
        return Ok(Json(Vec::new()));
    };

    let expression_query = ExpressionQuery {
        unit: query.unit,
        limit: query.limit,
        ..ExpressionQuery::default()
    };
    let points = expression_repository
        .gene_expression(&gene_record.gene.id, &expression_query)
        .into_iter()
        .map(|measurement| {
            let sample = expression_repository.sample(&measurement.run);
            let label = sample
                .as_ref()
                .map(expression_core::Sample::display_label)
                .unwrap_or_else(|| measurement.run.to_string());
            let primary_group = sample.as_ref().and_then(sample_primary_group);

            GeneExpressionPoint {
                gene_id: measurement.gene_id.to_string(),
                run: measurement.run.to_string(),
                label,
                primary_group,
                value: measurement.value,
                unit: measurement.unit,
            }
        })
        .collect();

    Ok(Json(points))
}

#[utoipa::path(
    get,
    path = "/v2/expression/clustergram",
    params(ExpressionClustergramQuery),
    responses(
        (status = 200, description = "Expression matrix with Rust-computed cluster ordering", body = ExpressionClustergramResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn clustergram(
    State(state): State<AppState>,
    Query(query): Query<ExpressionClustergramQuery>,
) -> Result<Json<ExpressionClustergramResponse>, ApiError> {
    let Some(expression_repository) = state.expression_repository.as_ref() else {
        return Ok(Json(ExpressionClustergramResponse::empty(
            query.assembly_accession,
            query.unit.unwrap_or(ExpressionUnit::Tpm),
        )));
    };

    let accession = AssemblyAccession::new(&query.assembly_accession)
        .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
    let unit = query.unit.unwrap_or(ExpressionUnit::Tpm);
    let gene_ids = parse_csv_gene_ids(&query.gene_ids)?;
    if gene_ids.is_empty() {
        return Err(ServiceError::InvalidRequest("geneIds must not be empty".to_owned()).into());
    }

    let genes = gene_ids
        .iter()
        .map(|gene_id| state.service.gene(gene_id.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    ensure_clustergram_genes_belong_to_assembly(&genes, &accession)?;

    let samples = expression_samples_for_query(expression_repository, &accession, &query)?;
    let runs = samples
        .iter()
        .map(|sample| sample.run().clone())
        .collect::<Vec<_>>();
    if runs.is_empty() {
        return Ok(Json(ExpressionClustergramResponse::empty(
            query.assembly_accession,
            unit,
        )));
    }

    let Some((matrix, genes)) = expression_clustergram_matrix(
        expression_repository,
        &accession,
        gene_ids,
        genes,
        &runs,
        unit,
        query.drop_missing_genes.unwrap_or(false),
    )?
    else {
        return Ok(Json(ExpressionClustergramResponse::empty(
            query.assembly_accession,
            unit,
        )));
    };

    Ok(Json(clustergram_response(matrix, genes, samples)))
}

fn clustergram_response(
    matrix: ExpressionMatrix,
    genes: Vec<genome_core::GeneRecord>,
    samples: Vec<expression_core::Sample>,
) -> ExpressionClustergramResponse {
    let values = matrix
        .values
        .iter()
        .copied()
        .map(finite_or_zero)
        .collect::<Vec<_>>();
    let row_dendrogram = cluster_dendrogram_rows(&values, matrix.gene_count(), matrix.run_count());
    let column_dendrogram =
        cluster_dendrogram_columns(&values, matrix.gene_count(), matrix.run_count());
    let row_order = row_dendrogram.leaf_order();
    let column_order = column_dendrogram.leaf_order();
    let z_scores = row_z_scores(&values, matrix.gene_count(), matrix.run_count());
    let genes = genes
        .into_iter()
        .map(|gene_record| ExpressionGeneLabel {
            gene_id: gene_record.gene.id.to_string(),
            label: gene_record
                .gene
                .symbol
                .or(gene_record.gene.locus_tag)
                .unwrap_or_else(|| gene_record.gene.id.to_string()),
        })
        .collect();
    let samples = samples
        .into_iter()
        .map(|sample| ExpressionSampleLabel {
            run: sample.run().to_string(),
            label: sample.display_label(),
            primary_group: sample_primary_group(&sample),
        })
        .collect();

    ExpressionClustergramResponse {
        assembly_accession: matrix.assembly_accession.to_string(),
        unit: matrix.unit,
        genes,
        samples,
        values,
        row_order,
        column_order,
        row_dendrogram,
        column_dendrogram,
        z_scores,
    }
}

fn sample_primary_group(sample: &expression_core::Sample) -> Option<String> {
    sample
        .metadata
        .primary_group
        .as_deref()
        .and_then(|key| sample.metadata_value(key))
        .map(ToOwned::to_owned)
}

fn parse_csv_gene_ids(value: &str) -> Result<Vec<genome_core::GeneId>, ApiError> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            genome_core::GeneId::new(value)
                .map_err(|error| ServiceError::InvalidRequest(error.to_string()).into())
        })
        .collect()
}

fn parse_csv_runs(value: &str) -> Result<Vec<SraRunAccession>, ApiError> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            SraRunAccession::new(value)
                .map_err(|error| ServiceError::InvalidRequest(error.to_string()).into())
        })
        .collect()
}

fn ensure_clustergram_genes_belong_to_assembly(
    genes: &[genome_core::GeneRecord],
    accession: &AssemblyAccession,
) -> Result<(), ApiError> {
    if genes
        .iter()
        .any(|gene_record| gene_record.gene.assembly_accession != *accession)
    {
        return Err(ServiceError::InvalidRequest(
            "all genes must belong to assemblyAccession".to_owned(),
        )
        .into());
    }
    Ok(())
}

fn expression_samples_for_query(
    repository: &AppExpressionRepository,
    accession: &AssemblyAccession,
    query: &ExpressionClustergramQuery,
) -> Result<Vec<expression_core::Sample>, ApiError> {
    let mut samples = repository.samples_for_assembly(accession);
    samples.sort_by_key(sample_sort_key);

    if let Some(runs) = query.runs.as_deref() {
        let requested_runs = parse_csv_runs(runs)?;
        let mut selected = Vec::with_capacity(requested_runs.len());
        for run in requested_runs {
            let Some(sample) = samples.iter().find(|sample| sample.run() == &run) else {
                return Err(
                    ServiceError::InvalidRequest(format!("unknown expression run: {run}")).into(),
                );
            };
            selected.push(sample.clone());
        }
        samples = selected;
    }

    if let Some(limit) = query.limit {
        samples.truncate(limit);
    }

    Ok(samples)
}

fn expression_clustergram_matrix(
    repository: &AppExpressionRepository,
    accession: &AssemblyAccession,
    gene_ids: Vec<GeneId>,
    genes: Vec<genome_core::GeneRecord>,
    runs: &[SraRunAccession],
    unit: ExpressionUnit,
    drop_missing_genes: bool,
) -> Result<Option<(ExpressionMatrix, Vec<genome_core::GeneRecord>)>, ApiError> {
    if let Some(matrix) = repository.expression_matrix(accession, &gene_ids, runs, unit) {
        return Ok(Some((matrix, genes)));
    }
    if !drop_missing_genes {
        return Err(ServiceError::InvalidRequest(
            "expression matrix is unavailable for the requested genes, runs, or unit".to_owned(),
        )
        .into());
    }

    let available = available_expression_genes(repository, accession, gene_ids, genes, runs, unit);
    let (available_gene_ids, available_genes): (Vec<_>, Vec<_>) = available.into_iter().unzip();
    if available_gene_ids.is_empty() {
        return Ok(None);
    }

    Ok(repository
        .expression_matrix(accession, &available_gene_ids, runs, unit)
        .map(|matrix| (matrix, available_genes)))
}

fn available_expression_genes(
    repository: &AppExpressionRepository,
    accession: &AssemblyAccession,
    gene_ids: Vec<GeneId>,
    genes: Vec<genome_core::GeneRecord>,
    runs: &[SraRunAccession],
    unit: ExpressionUnit,
) -> Vec<(GeneId, genome_core::GeneRecord)> {
    gene_ids
        .into_iter()
        .zip(genes)
        .filter(|(gene_id, _)| {
            repository
                .expression_matrix(accession, std::slice::from_ref(gene_id), runs, unit)
                .is_some()
        })
        .collect()
}

fn sample_sort_key(sample: &expression_core::Sample) -> (String, String, String) {
    (
        sample.metadata.sort_key.clone().unwrap_or_default(),
        sample.display_label(),
        sample.run().to_string(),
    )
}

fn finite_or_zero(value: f64) -> f64 {
    if value.is_finite() { value } else { 0.0 }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use expression_core::{Sample, SampleIdentity, SampleMetadata};
    use expression_store::ExpressionDataset;
    use genome_core::{
        AssemblyAccession, Gene, GeneRecord, HalfOpenRegion, Position0, SequenceName, Strand,
    };
    use std::collections::BTreeMap;

    const ASSEMBLY: &str = "GCA_test";
    const OTHER_ASSEMBLY: &str = "GCA_other";

    fn accession(value: &str) -> AssemblyAccession {
        AssemblyAccession::new(value).unwrap()
    }

    fn gene_id(value: &str) -> GeneId {
        GeneId::new(value).unwrap()
    }

    fn run(value: &str) -> SraRunAccession {
        SraRunAccession::new(value).unwrap()
    }

    fn sample(run_id: &str) -> Sample {
        Sample {
            identity: SampleIdentity {
                run: run(run_id),
                experiment: None,
                study: None,
                biosample: None,
                bioproject: None,
                assembly_accession: accession(ASSEMBLY),
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

    fn repository(matrices: Vec<ExpressionMatrix>) -> AppExpressionRepository {
        AppExpressionRepository::new(ExpressionDataset {
            assembly_accession: accession(ASSEMBLY),
            bioprojects: Vec::new(),
            samples: vec![sample("SRR000001"), sample("SRR000002")],
            matrices,
        })
        .unwrap()
    }

    fn gene_record(id: &str, assembly_accession: &str) -> GeneRecord {
        GeneRecord {
            gene: Gene {
                id: gene_id(id),
                assembly_accession: accession(assembly_accession),
                symbol: None,
                locus_tag: None,
                sequence_name: SequenceName::new("chr1").unwrap(),
                region: HalfOpenRegion::new(
                    SequenceName::new("chr1").unwrap(),
                    Position0::new(0),
                    Position0::new(10),
                )
                .unwrap(),
                strand: Strand::Forward,
                feature_type: "gene".to_owned(),
                annotations: Vec::new(),
                attributes: BTreeMap::new(),
            },
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
        }
    }

    fn query_with_runs(runs: Option<&str>) -> ExpressionClustergramQuery {
        ExpressionClustergramQuery {
            assembly_accession: ASSEMBLY.to_owned(),
            gene_ids: "Mp1g00010".to_owned(),
            unit: Some(ExpressionUnit::Tpm),
            runs: runs.map(ToOwned::to_owned),
            limit: None,
            drop_missing_genes: None,
        }
    }

    #[test]
    fn csv_parsers_ignore_empty_items_after_trimming() {
        assert_eq!(
            parse_csv_gene_ids(" Mp1g00010, ,Mp1g00020, ").unwrap(),
            vec![gene_id("Mp1g00010"), gene_id("Mp1g00020")]
        );
        assert_eq!(
            parse_csv_runs(" SRR000001, ,SRR000002, ").unwrap(),
            vec![run("SRR000001"), run("SRR000002")]
        );
    }

    #[test]
    fn clustergram_gene_assembly_validation_rejects_mismatches() {
        let matching = vec![gene_record("Mp1g00010", ASSEMBLY)];
        assert!(
            ensure_clustergram_genes_belong_to_assembly(&matching, &accession(ASSEMBLY)).is_ok()
        );

        let mixed = vec![
            gene_record("Mp1g00010", ASSEMBLY),
            gene_record("Mp9g00010", OTHER_ASSEMBLY),
        ];
        assert!(matches!(
            ensure_clustergram_genes_belong_to_assembly(&mixed, &accession(ASSEMBLY)),
            Err(ApiError::Service(ServiceError::InvalidRequest(_)))
        ));
    }

    #[test]
    fn expression_samples_for_query_selects_requested_runs_in_order() {
        let repository = repository(Vec::new());
        let samples = expression_samples_for_query(
            &repository,
            &accession(ASSEMBLY),
            &query_with_runs(Some("SRR000002,SRR000001")),
        )
        .unwrap();

        assert_eq!(
            samples
                .iter()
                .map(|sample| sample.run())
                .collect::<Vec<_>>(),
            vec![&run("SRR000002"), &run("SRR000001")]
        );
    }

    #[test]
    fn expression_clustergram_matrix_requires_drop_missing_for_missing_matrix() {
        let repository = repository(Vec::new());
        let genes = vec![gene_record("Mp1g00010", ASSEMBLY)];
        let gene_ids = vec![gene_id("Mp1g00010")];
        let runs = vec![run("SRR000001")];

        assert!(matches!(
            expression_clustergram_matrix(
                &repository,
                &accession(ASSEMBLY),
                gene_ids.clone(),
                genes.clone(),
                &runs,
                ExpressionUnit::Tpm,
                false,
            ),
            Err(ApiError::Service(ServiceError::InvalidRequest(_)))
        ));
        assert!(
            expression_clustergram_matrix(
                &repository,
                &accession(ASSEMBLY),
                gene_ids,
                genes,
                &runs,
                ExpressionUnit::Tpm,
                true,
            )
            .unwrap()
            .is_none()
        );
    }
}
