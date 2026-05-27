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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use genome_core::{
        AnnotationEvidence, AnnotationSource, Assembly, AssemblySource, GoTermAnnotation,
        HalfOpenRegion, InterProAnnotation, InterProId, KeggAnnotation, KeggEntryId, KogAnnotation,
        KogEntryId, NcbiFamAccession, NcbiFamAnnotation, PfamAccession, PfamAnnotation, Position0,
        SequenceName, Strand, TaxId, Taxon,
    };
    use std::collections::BTreeMap;
    use storage::FileGenomeRepository;

    const ASSEMBLY: &str = "GCA_test";
    const OTHER_ASSEMBLY: &str = "GCA_other";

    fn accession(value: &str) -> AssemblyAccession {
        AssemblyAccession::new(value).unwrap()
    }

    fn gene_id(value: &str) -> GeneId {
        GeneId::new(value).unwrap()
    }

    fn evidence() -> AnnotationEvidence {
        AnnotationEvidence::new(AnnotationSource::Manual)
    }

    fn gene(id: &str, assembly_accession: &str, annotations: Vec<FunctionalAnnotation>) -> Gene {
        Gene {
            id: gene_id(id),
            assembly_accession: accession(assembly_accession),
            symbol: Some(id.to_owned()),
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
            annotations,
            attributes: BTreeMap::new(),
        }
    }

    fn service_with_genes(genes: Vec<Gene>) -> AppService {
        let accession = accession(ASSEMBLY);
        let dataset = genome_core::GenomeDataset {
            taxon: Taxon {
                tax_id: TaxId::new(3197),
                scientific_name: "Marchantia polymorpha".to_owned(),
                common_name: None,
                rank: "species".to_owned(),
            },
            assembly: Assembly {
                accession,
                tax_id: TaxId::new(3197),
                name: "test".to_owned(),
                source: AssemblySource::Local,
                refget_checksum: None,
            },
            sequences: Vec::new(),
            genes,
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
            kegg_catalog: genome_core::KeggCatalog::default(),
        };
        service::GenomeService::new(FileGenomeRepository::new(dataset), None)
    }

    fn go_annotation(id: &str) -> FunctionalAnnotation {
        FunctionalAnnotation::GoTerm(GoTermAnnotation {
            term_id: genome_core::GoTermId::new(id).unwrap(),
            name: Some(id.to_owned()),
            namespace: Some(GoNamespace::BiologicalProcess),
            evidence: evidence(),
        })
    }

    fn all_annotation_kinds() -> Vec<(FunctionalAnnotation, EnrichmentAnnotationKind, &'static str)>
    {
        vec![
            (
                go_annotation("GO:0000001"),
                EnrichmentAnnotationKind::GoTerm,
                "GO:0000001",
            ),
            (
                FunctionalAnnotation::Pfam(PfamAnnotation {
                    accession: PfamAccession::new("PF00001").unwrap(),
                    name: Some("pfam".to_owned()),
                    interpro_id: None,
                    evidence: evidence(),
                }),
                EnrichmentAnnotationKind::Pfam,
                "PF00001",
            ),
            (
                FunctionalAnnotation::InterPro(InterProAnnotation {
                    interpro_id: InterProId::new("IPR000001").unwrap(),
                    name: Some("interpro".to_owned()),
                    evidence: evidence(),
                }),
                EnrichmentAnnotationKind::InterPro,
                "IPR000001",
            ),
            (
                FunctionalAnnotation::Kegg(KeggAnnotation::new(
                    KeggEntryId::new("K00001").unwrap(),
                    Some("kegg".to_owned()),
                    evidence(),
                )),
                EnrichmentAnnotationKind::Kegg,
                "K00001",
            ),
            (
                FunctionalAnnotation::Kog(KogAnnotation {
                    entry_id: KogEntryId::new("KOG0001").unwrap(),
                    name: Some("kog".to_owned()),
                    interpro_id: None,
                    evidence: evidence(),
                }),
                EnrichmentAnnotationKind::Kog,
                "KOG0001",
            ),
            (
                FunctionalAnnotation::NcbiFam(NcbiFamAnnotation {
                    accession: NcbiFamAccession::new("NF000001").unwrap(),
                    name: Some("ncbifam".to_owned()),
                    interpro_id: None,
                    evidence: evidence(),
                }),
                EnrichmentAnnotationKind::NcbiFam,
                "NF000001",
            ),
        ]
    }

    #[test]
    fn default_annotation_kinds_includes_every_supported_kind() {
        assert_eq!(
            default_enrichment_annotation_kinds(),
            vec![
                EnrichmentAnnotationKind::GoTerm,
                EnrichmentAnnotationKind::Pfam,
                EnrichmentAnnotationKind::InterPro,
                EnrichmentAnnotationKind::Kegg,
                EnrichmentAnnotationKind::Kog,
                EnrichmentAnnotationKind::NcbiFam,
            ]
        );
    }

    #[test]
    fn valid_enrichment_request_accepts_non_zero_limits() {
        let genes = vec![
            gene("Mp1g00010", ASSEMBLY, vec![go_annotation("GO:0000001")]),
            gene("Mp1g00020", ASSEMBLY, vec![go_annotation("GO:0000001")]),
        ];
        let service = service_with_genes(genes);
        let response = run_functional_enrichment(
            &service,
            EnrichmentAnalysisRequest {
                assembly_accession: ASSEMBLY.to_owned(),
                gene_ids: vec!["Mp1g00010".to_owned()],
                background_gene_ids: Some(vec!["Mp1g00010".to_owned(), "Mp1g00020".to_owned()]),
                annotation_kinds: Some(vec![EnrichmentAnnotationKind::GoTerm]),
                min_population_hits: Some(1),
                limit: Some(1),
            },
        )
        .unwrap();

        assert_eq!(response.study_size, 1);
        assert_eq!(response.population_size, 2);
        assert_eq!(response.tested_terms, 1);
        assert!(response.results.len() <= 1);
    }

    #[test]
    fn ensure_genes_belong_to_assembly_rejects_mixed_assemblies() {
        let genes = vec![
            gene("Mp1g00010", ASSEMBLY, Vec::new()),
            gene("Mp9g00010", OTHER_ASSEMBLY, Vec::new()),
        ];

        assert!(ensure_genes_belong_to_assembly(&genes[..1], ASSEMBLY, "geneIds").is_ok());
        assert!(matches!(
            ensure_genes_belong_to_assembly(&genes, ASSEMBLY, "geneIds"),
            Err(ApiError::Service(ServiceError::InvalidRequest(_)))
        ));
    }

    #[test]
    fn genes_for_assembly_uses_unbounded_search_and_filters_assembly() {
        let mut genes = (0..55)
            .map(|index| gene(&format!("Mp1g{index:05}"), ASSEMBLY, Vec::new()))
            .collect::<Vec<_>>();
        genes.push(gene("Mp9g00001", OTHER_ASSEMBLY, Vec::new()));
        let service = service_with_genes(genes);

        let result = genes_for_assembly(&service, ASSEMBLY);

        assert_eq!(result.len(), 55);
        assert!(
            result
                .iter()
                .all(|gene| gene.assembly_accession.as_str() == ASSEMBLY)
        );
    }

    #[test]
    fn enrichment_terms_respect_selected_annotation_kinds() {
        for (annotation, kind, expected_id) in all_annotation_kinds() {
            let selected = HashSet::from([kind]);
            let term = enrichment_term_for_annotation(&annotation, &selected).unwrap();
            assert_eq!(term.kind, kind);
            assert_eq!(term.id, expected_id);

            let unselected = enrichment_term_for_annotation(&annotation, &HashSet::new());
            assert!(unselected.is_none());
        }
    }
}
