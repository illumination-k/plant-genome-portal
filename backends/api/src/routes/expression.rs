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
    if genes
        .iter()
        .any(|gene_record| gene_record.gene.assembly_accession != accession)
    {
        return Err(ServiceError::InvalidRequest(
            "all genes must belong to assemblyAccession".to_owned(),
        )
        .into());
    }

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
