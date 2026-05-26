use axum::{Json, extract::State};
use enrichment_core::{EnrichmentInput, EnrichmentOptions, run_enrichment};
use genome_core::{AssemblyAccession, FunctionalAnnotation, Gene, GeneId, GeneSearch, GoNamespace};
use serde::{Deserialize, Serialize};
use service::ServiceError;
use std::collections::{HashMap, HashSet};
use utoipa::ToSchema;

use crate::{ApiError, AppService, AppState};

#[utoipa::path(
    post,
    path = "/v2/analysis/enrichment",
    request_body = EnrichmentAnalysisRequest,
    responses(
        (status = 200, description = "Functional annotation over-representation analysis", body = EnrichmentAnalysisResponse),
        (status = 404, description = "Assembly or gene not found", body = crate::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::ErrorResponse),
    )
)]
pub(crate) async fn analysis(
    State(state): State<AppState>,
    Json(request): Json<EnrichmentAnalysisRequest>,
) -> Result<Json<EnrichmentAnalysisResponse>, ApiError> {
    Ok(Json(run_functional_enrichment(&state.service, request)?))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EnrichmentAnalysisRequest {
    assembly_accession: String,
    gene_ids: Vec<String>,
    background_gene_ids: Option<Vec<String>>,
    annotation_kinds: Option<Vec<EnrichmentAnnotationKind>>,
    min_population_hits: Option<u64>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EnrichmentAnnotationKind {
    GoTerm,
    Pfam,
    InterPro,
    Kegg,
    Kog,
    NcbiFam,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EnrichmentAnalysisResponse {
    assembly_accession: String,
    study_size: u64,
    population_size: u64,
    tested_terms: usize,
    results: Vec<EnrichmentTermResult>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EnrichmentTermResult {
    term: EnrichmentTerm,
    study_hits: u64,
    study_size: u64,
    population_hits: u64,
    population_size: u64,
    fold_enrichment: Option<f64>,
    p_value: f64,
    q_value: f64,
    study_gene_ids: Vec<GeneId>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EnrichmentTerm {
    kind: EnrichmentAnnotationKind,
    id: String,
    name: Option<String>,
    namespace: Option<GoNamespace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EnrichmentTermKey {
    kind: EnrichmentAnnotationKind,
    id: String,
}

type EnrichmentTermItems = Vec<(EnrichmentTermKey, HashSet<GeneId>)>;
type EnrichmentTermMetadata = HashMap<EnrichmentTermKey, EnrichmentTerm>;

fn run_functional_enrichment(
    service: &AppService,
    request: EnrichmentAnalysisRequest,
) -> Result<EnrichmentAnalysisResponse, ApiError> {
    let accession = AssemblyAccession::new(&request.assembly_accession)
        .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
    let assembly_accession = accession.into_string();
    service.assembly(&assembly_accession)?;

    let study_genes = resolve_gene_ids(service, &request.gene_ids, "geneIds")?;
    if study_genes.is_empty() {
        return Err(ServiceError::InvalidRequest("geneIds must not be empty".to_owned()).into());
    }
    ensure_genes_belong_to_assembly(&study_genes, &assembly_accession, "geneIds")?;

    let population_genes = match request.background_gene_ids.as_ref() {
        Some(gene_ids) => {
            let genes = resolve_gene_ids(service, gene_ids, "backgroundGeneIds")?;
            if genes.is_empty() {
                return Err(ServiceError::InvalidRequest(
                    "backgroundGeneIds must not be empty when provided".to_owned(),
                )
                .into());
            }
            ensure_genes_belong_to_assembly(&genes, &assembly_accession, "backgroundGeneIds")?;
            genes
        }
        None => genes_for_assembly(service, &assembly_accession),
    };

    let kinds = request
        .annotation_kinds
        .unwrap_or_else(default_enrichment_annotation_kinds);
    if kinds.is_empty() {
        return Err(
            ServiceError::InvalidRequest("annotationKinds must not be empty".to_owned()).into(),
        );
    }
    let min_population_hits = request.min_population_hits.unwrap_or(2);
    if min_population_hits == 0 {
        return Err(ServiceError::InvalidRequest(
            "minPopulationHits must be greater than zero".to_owned(),
        )
        .into());
    }
    let limit = request.limit.unwrap_or(50);
    if limit == 0 {
        return Err(
            ServiceError::InvalidRequest("limit must be greater than zero".to_owned()).into(),
        );
    }

    let study: HashSet<GeneId> = study_genes.into_iter().map(|gene| gene.id).collect();
    let population: HashSet<GeneId> = population_genes
        .iter()
        .map(|gene| gene.id.clone())
        .collect();
    if population.is_empty() {
        return Err(ServiceError::InvalidRequest(
            "population has no genes for the selected assembly".to_owned(),
        )
        .into());
    }

    let (terms, term_metadata) = build_enrichment_terms(&population_genes, &kinds);
    let results = run_enrichment(
        EnrichmentInput {
            study: &study,
            population: &population,
            term_to_items: &terms,
        },
        EnrichmentOptions {
            min_population_hits,
        },
    )
    .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;

    let response_results =
        enrichment_response_results(results, &terms, &term_metadata, &study, limit);

    Ok(EnrichmentAnalysisResponse {
        assembly_accession,
        study_size: study.intersection(&population).count() as u64,
        population_size: population.len() as u64,
        tested_terms: terms.len(),
        results: response_results,
    })
}

fn enrichment_response_results(
    results: Vec<enrichment_core::EnrichmentResult<EnrichmentTermKey>>,
    terms: &EnrichmentTermItems,
    metadata: &EnrichmentTermMetadata,
    study: &HashSet<GeneId>,
    limit: usize,
) -> Vec<EnrichmentTermResult> {
    let genes_by_term = terms.iter().cloned().collect::<HashMap<_, _>>();
    results
        .into_iter()
        .take(limit)
        .map(|result| enrichment_response_result(result, &genes_by_term, metadata, study))
        .collect()
}

fn enrichment_response_result(
    result: enrichment_core::EnrichmentResult<EnrichmentTermKey>,
    genes_by_term: &HashMap<EnrichmentTermKey, HashSet<GeneId>>,
    metadata: &EnrichmentTermMetadata,
    study: &HashSet<GeneId>,
) -> EnrichmentTermResult {
    let mut study_gene_ids = genes_by_term
        .get(&result.term)
        .map(|genes| {
            genes
                .iter()
                .filter(|gene_id| study.contains(*gene_id))
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    study_gene_ids.sort();

    EnrichmentTermResult {
        term: metadata
            .get(&result.term)
            .cloned()
            .unwrap_or_else(|| fallback_enrichment_term(&result.term)),
        study_hits: result.study_hits,
        study_size: result.study_size,
        population_hits: result.population_hits,
        population_size: result.population_size,
        fold_enrichment: result.fold_enrichment,
        p_value: result.p_value,
        q_value: result.q_value,
        study_gene_ids,
    }
}

fn fallback_enrichment_term(key: &EnrichmentTermKey) -> EnrichmentTerm {
    EnrichmentTerm {
        kind: key.kind,
        id: key.id.clone(),
        name: None,
        namespace: None,
    }
}

fn default_enrichment_annotation_kinds() -> Vec<EnrichmentAnnotationKind> {
    vec![
        EnrichmentAnnotationKind::GoTerm,
        EnrichmentAnnotationKind::Pfam,
        EnrichmentAnnotationKind::InterPro,
        EnrichmentAnnotationKind::Kegg,
        EnrichmentAnnotationKind::Kog,
        EnrichmentAnnotationKind::NcbiFam,
    ]
}

fn resolve_gene_ids(
    service: &AppService,
    gene_ids: &[String],
    field_name: &str,
) -> Result<Vec<Gene>, ApiError> {
    let mut genes = Vec::new();
    let mut seen = HashSet::new();
    for raw_gene_id in gene_ids {
        let gene_id = raw_gene_id.trim();
        if gene_id.is_empty() {
            continue;
        }
        let gene = service.gene(gene_id)?.gene;
        if seen.insert(gene.id.clone()) {
            genes.push(gene);
        }
    }
    if genes.is_empty() {
        return Err(ServiceError::InvalidRequest(format!("{field_name} must not be empty")).into());
    }
    Ok(genes)
}

fn ensure_genes_belong_to_assembly(
    genes: &[Gene],
    assembly_accession: &str,
    field_name: &str,
) -> Result<(), ApiError> {
    if genes
        .iter()
        .all(|gene| gene.assembly_accession.as_str() == assembly_accession)
    {
        return Ok(());
    }
    Err(ServiceError::InvalidRequest(format!(
        "all genes in {field_name} must belong to assemblyAccession"
    ))
    .into())
}

fn genes_for_assembly(service: &AppService, assembly_accession: &str) -> Vec<Gene> {
    service
        .search_genes(GeneSearch {
            limit: Some(usize::MAX),
            ..GeneSearch::default()
        })
        .into_iter()
        .filter(|gene| gene.assembly_accession.as_str() == assembly_accession)
        .collect()
}

fn build_enrichment_terms(
    genes: &[Gene],
    kinds: &[EnrichmentAnnotationKind],
) -> (EnrichmentTermItems, EnrichmentTermMetadata) {
    let selected: HashSet<EnrichmentAnnotationKind> = kinds.iter().copied().collect();
    let mut term_to_items: HashMap<EnrichmentTermKey, HashSet<GeneId>> = HashMap::new();
    let mut metadata: HashMap<EnrichmentTermKey, EnrichmentTerm> = HashMap::new();

    for gene in genes {
        let mut seen_for_gene = HashSet::new();
        for annotation in &gene.annotations {
            let Some(term) = enrichment_term_for_annotation(annotation, &selected) else {
                continue;
            };
            let key = EnrichmentTermKey {
                kind: term.kind,
                id: term.id.clone(),
            };
            metadata.entry(key.clone()).or_insert(term);
            if seen_for_gene.insert(key.clone()) {
                term_to_items
                    .entry(key)
                    .or_default()
                    .insert(gene.id.clone());
            }
        }
    }

    let mut terms = term_to_items.into_iter().collect::<Vec<_>>();
    terms.sort_by(|(left, _), (right, _)| {
        enrichment_kind_rank(left.kind)
            .cmp(&enrichment_kind_rank(right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });
    (terms, metadata)
}

fn enrichment_term_for_annotation(
    annotation: &FunctionalAnnotation,
    selected: &HashSet<EnrichmentAnnotationKind>,
) -> Option<EnrichmentTerm> {
    match annotation {
        FunctionalAnnotation::GoTerm(go)
            if selected.contains(&EnrichmentAnnotationKind::GoTerm) =>
        {
            Some(EnrichmentTerm {
                kind: EnrichmentAnnotationKind::GoTerm,
                id: go.term_id.as_str().to_owned(),
                name: go.name.clone(),
                namespace: go.namespace,
            })
        }
        FunctionalAnnotation::Pfam(pfam) if selected.contains(&EnrichmentAnnotationKind::Pfam) => {
            Some(EnrichmentTerm {
                kind: EnrichmentAnnotationKind::Pfam,
                id: pfam.accession.as_str().to_owned(),
                name: pfam.name.clone(),
                namespace: None,
            })
        }
        FunctionalAnnotation::InterPro(interpro)
            if selected.contains(&EnrichmentAnnotationKind::InterPro) =>
        {
            Some(EnrichmentTerm {
                kind: EnrichmentAnnotationKind::InterPro,
                id: interpro.interpro_id.as_str().to_owned(),
                name: interpro.name.clone(),
                namespace: None,
            })
        }
        FunctionalAnnotation::Kegg(kegg) if selected.contains(&EnrichmentAnnotationKind::Kegg) => {
            Some(EnrichmentTerm {
                kind: EnrichmentAnnotationKind::Kegg,
                id: kegg.entry_id.as_str().to_owned(),
                name: kegg.name.clone(),
                namespace: None,
            })
        }
        FunctionalAnnotation::Kog(kog) if selected.contains(&EnrichmentAnnotationKind::Kog) => {
            Some(EnrichmentTerm {
                kind: EnrichmentAnnotationKind::Kog,
                id: kog.entry_id.as_str().to_owned(),
                name: kog.name.clone(),
                namespace: None,
            })
        }
        FunctionalAnnotation::NcbiFam(ncbi_fam)
            if selected.contains(&EnrichmentAnnotationKind::NcbiFam) =>
        {
            Some(EnrichmentTerm {
                kind: EnrichmentAnnotationKind::NcbiFam,
                id: ncbi_fam.accession.as_str().to_owned(),
                name: ncbi_fam.name.clone(),
                namespace: None,
            })
        }
        _ => None,
    }
}

fn enrichment_kind_rank(kind: EnrichmentAnnotationKind) -> u8 {
    match kind {
        EnrichmentAnnotationKind::GoTerm => 0,
        EnrichmentAnnotationKind::Pfam => 1,
        EnrichmentAnnotationKind::InterPro => 2,
        EnrichmentAnnotationKind::Kegg => 3,
        EnrichmentAnnotationKind::Kog => 4,
        EnrichmentAnnotationKind::NcbiFam => 5,
    }
}
