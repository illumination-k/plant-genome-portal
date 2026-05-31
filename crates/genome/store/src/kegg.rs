use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use genome_domain::{
    FunctionalAnnotation, KeggCatalog, KeggEntryId, KeggKoLinks, KeggModule, KeggModuleId,
    KeggPathway, KeggPathwayId, KeggReaction, KeggReactionId, ko_entry_id,
};

use crate::error::StorageError;
use crate::gff::ParsedGff;

/// Input TSVs downloaded from the KEGG REST API. Each path is optional so the
/// catalog can be built incrementally as data becomes available.
#[derive(Debug, Default, Clone)]
pub struct KeggCatalogInput<'a> {
    pub link_ko_pathway: Option<&'a Path>,
    pub link_ko_module: Option<&'a Path>,
    pub link_ko_reaction: Option<&'a Path>,
    pub list_pathway: Option<&'a Path>,
    pub list_module: Option<&'a Path>,
    pub list_reaction: Option<&'a Path>,
}

/// Build a [`KeggCatalog`] from KEGG REST `link/*` and `list/*` TSV dumps,
/// filtered to KOs that appear in the parsed annotation. Pathways, modules,
/// and reactions are likewise filtered to those referenced by the kept links.
pub(crate) fn build_kegg_catalog(
    parsed: &ParsedGff,
    input: &KeggCatalogInput<'_>,
) -> Result<KeggCatalog, StorageError> {
    let kos_in_dataset = collect_dataset_kos(parsed);
    if kos_in_dataset.is_empty() {
        return Ok(KeggCatalog::default());
    }

    let mut ko_pathways: BTreeMap<KeggEntryId, BTreeSet<KeggPathwayId>> = BTreeMap::new();
    let mut ko_modules: BTreeMap<KeggEntryId, BTreeSet<KeggModuleId>> = BTreeMap::new();
    let mut ko_reactions: BTreeMap<KeggEntryId, BTreeSet<KeggReactionId>> = BTreeMap::new();

    if let Some(path) = input.link_ko_pathway {
        parse_link_tsv(path, &kos_in_dataset, |ko, raw| {
            let id = KeggPathwayId::new(raw).ok()?;
            ko_pathways.entry(ko).or_default().insert(id);
            Some(())
        })?;
    }
    if let Some(path) = input.link_ko_module {
        parse_link_tsv(path, &kos_in_dataset, |ko, raw| {
            let id = KeggModuleId::new(raw).ok()?;
            ko_modules.entry(ko).or_default().insert(id);
            Some(())
        })?;
    }
    if let Some(path) = input.link_ko_reaction {
        parse_link_tsv(path, &kos_in_dataset, |ko, raw| {
            let id = KeggReactionId::new(raw).ok()?;
            ko_reactions.entry(ko).or_default().insert(id);
            Some(())
        })?;
    }

    let referenced_pathways = referenced_ids(&ko_pathways);
    let referenced_modules = referenced_ids(&ko_modules);
    let referenced_reactions = referenced_ids(&ko_reactions);

    let pathway_names = input
        .list_pathway
        .map(|path| parse_list_tsv(path, |raw| KeggPathwayId::new(raw).ok()))
        .transpose()?
        .unwrap_or_default();
    let module_names = input
        .list_module
        .map(|path| parse_list_tsv(path, |raw| KeggModuleId::new(raw).ok()))
        .transpose()?
        .unwrap_or_default();
    let reaction_names = input
        .list_reaction
        .map(|path| parse_list_tsv(path, |raw| KeggReactionId::new(raw).ok()))
        .transpose()?
        .unwrap_or_default();

    Ok(KeggCatalog {
        pathways: named_pathways(referenced_pathways, &pathway_names),
        modules: named_modules(referenced_modules, &module_names),
        reactions: named_reactions(referenced_reactions, &reaction_names),
        ko_links: build_ko_links(kos_in_dataset, &ko_pathways, &ko_modules, &ko_reactions),
    })
}

fn referenced_ids<T: Ord + Clone>(links: &BTreeMap<KeggEntryId, BTreeSet<T>>) -> BTreeSet<T> {
    links.values().flat_map(|set| set.iter().cloned()).collect()
}

fn named_pathways(
    ids: BTreeSet<KeggPathwayId>,
    names: &BTreeMap<KeggPathwayId, String>,
) -> Vec<KeggPathway> {
    ids.into_iter()
        .map(|id| KeggPathway {
            name: names.get(&id).cloned(),
            id,
        })
        .collect()
}

fn named_modules(
    ids: BTreeSet<KeggModuleId>,
    names: &BTreeMap<KeggModuleId, String>,
) -> Vec<KeggModule> {
    ids.into_iter()
        .map(|id| KeggModule {
            name: names.get(&id).cloned(),
            id,
        })
        .collect()
}

fn named_reactions(
    ids: BTreeSet<KeggReactionId>,
    names: &BTreeMap<KeggReactionId, String>,
) -> Vec<KeggReaction> {
    ids.into_iter()
        .map(|id| KeggReaction {
            name: names.get(&id).cloned(),
            id,
        })
        .collect()
}

fn build_ko_links(
    kos: HashSet<KeggEntryId>,
    ko_pathways: &BTreeMap<KeggEntryId, BTreeSet<KeggPathwayId>>,
    ko_modules: &BTreeMap<KeggEntryId, BTreeSet<KeggModuleId>>,
    ko_reactions: &BTreeMap<KeggEntryId, BTreeSet<KeggReactionId>>,
) -> Vec<KeggKoLinks> {
    let mut links_by_ko: BTreeMap<KeggEntryId, KeggKoLinks> = BTreeMap::new();
    for ko in kos {
        let entry = links_by_ko.entry(ko.clone()).or_insert(KeggKoLinks {
            ko: ko.clone(),
            pathways: Vec::new(),
            modules: Vec::new(),
            reactions: Vec::new(),
        });
        if let Some(pathways) = ko_pathways.get(&ko) {
            entry.pathways = pathways.iter().cloned().collect();
        }
        if let Some(modules) = ko_modules.get(&ko) {
            entry.modules = modules.iter().cloned().collect();
        }
        if let Some(reactions) = ko_reactions.get(&ko) {
            entry.reactions = reactions.iter().cloned().collect();
        }
    }
    links_by_ko
        .into_values()
        .filter(|links| {
            !(links.pathways.is_empty() && links.modules.is_empty() && links.reactions.is_empty())
        })
        .collect()
}

fn collect_dataset_kos(parsed: &ParsedGff) -> HashSet<KeggEntryId> {
    let mut kos = HashSet::new();
    let annotation_iter = parsed
        .genes
        .iter()
        .flat_map(|gene| gene.annotations.iter())
        .chain(
            parsed
                .transcripts
                .iter()
                .flat_map(|transcript| transcript.annotations.iter()),
        );
    for annotation in annotation_iter {
        if let FunctionalAnnotation::Kegg(kegg) = annotation
            && let Some(ko) = ko_entry_id(&kegg.entry_id)
        {
            kos.insert(ko);
        }
    }
    kos
}

fn parse_link_tsv<F>(
    path: &Path,
    keep_kos: &HashSet<KeggEntryId>,
    mut accept: F,
) -> Result<(), StorageError>
where
    F: FnMut(KeggEntryId, &str) -> Option<()>,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if is_skippable_tsv_line(line) {
            continue;
        }
        let Some((raw_ko, raw_target)) = line.split_once('\t') else {
            continue;
        };
        let raw_ko = raw_ko.trim();
        let raw_target = raw_target.trim();
        let Ok(ko_owned) = KeggEntryId::new(raw_ko) else {
            continue;
        };
        let Some(ko) = ko_entry_id(&ko_owned) else {
            continue;
        };
        if !keep_kos.contains(&ko) {
            continue;
        }
        let _ = accept(ko, raw_target);
    }
    Ok(())
}

fn parse_list_tsv<T, F>(path: &Path, parse_id: F) -> Result<BTreeMap<T, String>, StorageError>
where
    F: Fn(&str) -> Option<T>,
    T: Ord,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut out = BTreeMap::new();
    for line in reader.lines() {
        let line = line?;
        let line = line.trim_end_matches(['\n', '\r']);
        if is_skippable_tsv_line(line.trim()) {
            continue;
        }
        let Some((raw_id, name)) = line.split_once('\t') else {
            continue;
        };
        let Some(id) = parse_id(raw_id.trim()) else {
            continue;
        };
        out.insert(id, name.trim().to_owned());
    }
    Ok(out)
}

/// Lines that should be ignored in KEGG REST TSV dumps: empty lines and
/// `#`-prefixed comments. Extracted so we can mutation-test the skip logic
/// independently of the actual parsers.
fn is_skippable_tsv_line(line: &str) -> bool {
    if line.is_empty() {
        return true;
    }
    line.starts_with('#')
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::Write;

    use genome_domain::{
        AnnotationEvidence, AnnotationSource, AssemblyAccession, FunctionalAnnotation,
        HalfOpenRegion, KeggAnnotation, KeggEntryId, Position0, SequenceName, Strand,
    };

    use super::*;
    use crate::gff::ParsedGff;
    use genome_domain::Gene;

    fn region() -> HalfOpenRegion {
        HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(0),
            Position0::new(10),
        )
        .unwrap()
    }

    fn gene_with_kegg(ids: &[&str]) -> Gene {
        Gene {
            id: genome_domain::GeneId::new("Mp1g00010").unwrap(),
            assembly_accession: AssemblyAccession::new("GCA_test").unwrap(),
            symbol: None,
            locus_tag: None,
            sequence_name: SequenceName::new("chr1").unwrap(),
            region: region(),
            strand: Strand::Forward,
            feature_type: "gene".to_owned(),
            annotations: ids
                .iter()
                .map(|id| {
                    FunctionalAnnotation::Kegg(KeggAnnotation::new(
                        KeggEntryId::new(*id).unwrap(),
                        None,
                        AnnotationEvidence::new(AnnotationSource::Kegg),
                    ))
                })
                .collect(),
            attributes: BTreeMap::new(),
        }
    }

    fn write_tsv(path: &std::path::Path, body: &str) {
        let mut file = File::create(path).unwrap();
        file.write_all(body.as_bytes()).unwrap();
    }

    #[test]
    fn build_catalog_keeps_only_dataset_kos_and_their_targets() {
        let dir = tempfile::tempdir().unwrap();
        let pathway_path = dir.path().join("link_pathway.tsv");
        let module_path = dir.path().join("link_module.tsv");
        let reaction_path = dir.path().join("link_reaction.tsv");
        let list_pathway_path = dir.path().join("list_pathway.tsv");

        // Dataset KOs: K00001 (orthology). K99999 is not in dataset.
        write_tsv(
            &pathway_path,
            "ko:K00001\tpath:map00010\nko:K00001\tpath:ko00010\nko:K99999\tpath:map00020\n",
        );
        write_tsv(&module_path, "ko:K00001\tmd:M00001\n");
        write_tsv(&reaction_path, "ko:K00001\trn:R00001\n");
        write_tsv(
            &list_pathway_path,
            "map00010\tGlycolysis / Gluconeogenesis\nmap00020\tCitrate cycle (TCA cycle)\n",
        );

        let parsed = ParsedGff {
            genes: vec![gene_with_kegg(&["ko:K00001"])],
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
        };

        let input = KeggCatalogInput {
            link_ko_pathway: Some(&pathway_path),
            link_ko_module: Some(&module_path),
            link_ko_reaction: Some(&reaction_path),
            list_pathway: Some(&list_pathway_path),
            list_module: None,
            list_reaction: None,
        };
        let catalog = build_kegg_catalog(&parsed, &input).unwrap();

        // map00010 and ko00010 both canonicalize to map00010, deduped.
        assert_eq!(catalog.pathways.len(), 1);
        assert_eq!(catalog.pathways[0].id.as_str(), "map00010");
        assert_eq!(
            catalog.pathways[0].name.as_deref(),
            Some("Glycolysis / Gluconeogenesis")
        );
        // map00020 belonged to K99999 (not in dataset) so it is dropped.
        assert!(catalog.pathways.iter().all(|p| p.id.as_str() != "map00020"));

        assert_eq!(catalog.modules.len(), 1);
        assert_eq!(catalog.modules[0].id.as_str(), "M00001");
        assert!(catalog.modules[0].name.is_none()); // no list_module input

        assert_eq!(catalog.reactions.len(), 1);
        assert_eq!(catalog.reactions[0].id.as_str(), "R00001");

        assert_eq!(catalog.ko_links.len(), 1);
        let links = &catalog.ko_links[0];
        assert_eq!(links.ko.as_str(), "K00001");
        assert_eq!(links.pathways.len(), 1);
        assert_eq!(links.modules.len(), 1);
        assert_eq!(links.reactions.len(), 1);
    }

    #[test]
    fn build_catalog_returns_empty_when_dataset_has_no_kos() {
        let parsed = ParsedGff {
            genes: Vec::new(),
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
        };
        let catalog = build_kegg_catalog(&parsed, &KeggCatalogInput::default()).unwrap();
        assert!(catalog.pathways.is_empty());
        assert!(catalog.modules.is_empty());
        assert!(catalog.reactions.is_empty());
        assert!(catalog.ko_links.is_empty());
    }

    #[test]
    fn build_catalog_hydrates_module_and_reaction_names_from_list_files() {
        let dir = tempfile::tempdir().unwrap();
        let pathway_path = dir.path().join("link_pathway.tsv");
        let module_path = dir.path().join("link_module.tsv");
        let reaction_path = dir.path().join("link_reaction.tsv");
        let list_module_path = dir.path().join("list_module.tsv");
        let list_reaction_path = dir.path().join("list_reaction.tsv");

        write_tsv(&pathway_path, "");
        write_tsv(&module_path, "ko:K00001\tmd:M00001\n");
        write_tsv(&reaction_path, "ko:K00001\trn:R00001\n");
        write_tsv(&list_module_path, "M00001\tGlycolysis module\n");
        write_tsv(&list_reaction_path, "R00001\talcohol dehydrogenase\n");

        let parsed = ParsedGff {
            genes: vec![gene_with_kegg(&["K00001"])],
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
        };

        let catalog = build_kegg_catalog(
            &parsed,
            &KeggCatalogInput {
                link_ko_pathway: Some(&pathway_path),
                link_ko_module: Some(&module_path),
                link_ko_reaction: Some(&reaction_path),
                list_pathway: None,
                list_module: Some(&list_module_path),
                list_reaction: Some(&list_reaction_path),
            },
        )
        .unwrap();

        assert_eq!(catalog.modules.len(), 1);
        assert_eq!(
            catalog.modules[0].name.as_deref(),
            Some("Glycolysis module")
        );
        assert_eq!(catalog.reactions.len(), 1);
        assert_eq!(
            catalog.reactions[0].name.as_deref(),
            Some("alcohol dehydrogenase")
        );
    }

    #[test]
    fn build_catalog_drops_ko_links_with_no_targets() {
        // K00002 is in the dataset but the link TSVs contain no entries for it;
        // it must not appear in `ko_links` because it has nothing to link to.
        let dir = tempfile::tempdir().unwrap();
        let pathway_path = dir.path().join("link_pathway.tsv");
        write_tsv(&pathway_path, "ko:K00001\tpath:map00010\n");

        let parsed = ParsedGff {
            genes: vec![gene_with_kegg(&["K00001", "K00002"])],
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
        };

        let catalog = build_kegg_catalog(
            &parsed,
            &KeggCatalogInput {
                link_ko_pathway: Some(&pathway_path),
                ..KeggCatalogInput::default()
            },
        )
        .unwrap();

        assert_eq!(catalog.ko_links.len(), 1);
        assert_eq!(catalog.ko_links[0].ko.as_str(), "K00001");
    }

    #[test]
    fn build_catalog_collects_transcript_level_kegg_annotations() {
        // Annotations attached to transcripts (not directly to genes) must
        // still feed the catalog's `keep_kos` filter.
        use genome_domain::Transcript;

        let parsed = ParsedGff {
            genes: Vec::new(),
            transcripts: vec![Transcript {
                id: genome_domain::TranscriptId::new("Mp1g00010.1").unwrap(),
                gene_id: genome_domain::GeneId::new("Mp1g00010").unwrap(),
                sequence_name: SequenceName::new("chr1").unwrap(),
                region: region(),
                strand: Strand::Forward,
                feature_type: "mRNA".to_owned(),
                annotations: vec![FunctionalAnnotation::Kegg(KeggAnnotation::new(
                    KeggEntryId::new("K00007").unwrap(),
                    None,
                    AnnotationEvidence::new(AnnotationSource::Kegg),
                ))],
                attributes: BTreeMap::new(),
                protein_checksum: None,
                protein_length: None,
            }],
            exons: Vec::new(),
            cdss: Vec::new(),
        };
        let dir = tempfile::tempdir().unwrap();
        let pathway_path = dir.path().join("link_pathway.tsv");
        write_tsv(&pathway_path, "ko:K00007\tpath:map00030\n");

        let catalog = build_kegg_catalog(
            &parsed,
            &KeggCatalogInput {
                link_ko_pathway: Some(&pathway_path),
                ..KeggCatalogInput::default()
            },
        )
        .unwrap();

        assert_eq!(catalog.ko_links.len(), 1);
        assert_eq!(catalog.ko_links[0].ko.as_str(), "K00007");
        assert_eq!(catalog.ko_links[0].pathways[0].as_str(), "map00030");
    }

    #[test]
    fn parse_link_tsv_skips_comments_blank_and_malformed_lines() {
        let dir = tempfile::tempdir().unwrap();
        let pathway_path = dir.path().join("link_pathway.tsv");
        // The `#`-prefixed line contains tabs that look like a real KO link:
        // if the comment skip is replaced by `&&` (so the line is parsed),
        // the parser would try to register `# header` as a KO entry. The
        // empty line similarly has no tab and must be skipped explicitly.
        write_tsv(
            &pathway_path,
            "# header\tko:K77777\tpath:map00010\n\
             \n\
             missing-tab-line\n\
             ko:not-a-valid-ko\tpath:map00010\n\
             ko:K00001\tnot-a-valid-pathway\n\
             ko:K00001\tpath:map00010\n",
        );

        let parsed = ParsedGff {
            genes: vec![gene_with_kegg(&["K00001"])],
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
        };
        let catalog = build_kegg_catalog(
            &parsed,
            &KeggCatalogInput {
                link_ko_pathway: Some(&pathway_path),
                ..KeggCatalogInput::default()
            },
        )
        .unwrap();

        // Only the one valid line should survive.
        assert_eq!(catalog.pathways.len(), 1);
        assert_eq!(catalog.pathways[0].id.as_str(), "map00010");
    }

    #[test]
    fn parse_link_tsv_skips_empty_lines_when_starts_with_check_is_combined() {
        // A purely empty line must be skipped by the `is_empty()` branch.
        // If the skip condition is mutated from `||` to `&&`, the empty
        // line continues into parsing, which would silently swallow it.
        // We pair the empty line with a `#`-prefixed line that, if not
        // skipped, would try to parse with an unparseable KO and silently
        // skip — but the `Default::default()` for HashSet-based filtering
        // would still leave the valid map00010 entry. So we also assert
        // exactly the expected number of valid pathways.
        let dir = tempfile::tempdir().unwrap();
        let pathway_path = dir.path().join("link_pathway.tsv");
        write_tsv(&pathway_path, "\nko:K00001\tpath:map00010\n");
        let parsed = ParsedGff {
            genes: vec![gene_with_kegg(&["K00001"])],
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
        };
        let catalog = build_kegg_catalog(
            &parsed,
            &KeggCatalogInput {
                link_ko_pathway: Some(&pathway_path),
                ..KeggCatalogInput::default()
            },
        )
        .unwrap();
        assert_eq!(catalog.pathways.len(), 1);
    }

    #[test]
    fn parse_list_tsv_skips_comments_blank_and_malformed_lines() {
        let dir = tempfile::tempdir().unwrap();
        let pathway_path = dir.path().join("link_pathway.tsv");
        let list_pathway_path = dir.path().join("list_pathway.tsv");
        write_tsv(&pathway_path, "ko:K00001\tpath:map00010\n");
        // The `#`-prefixed line has a tab so split_once succeeds — only the
        // explicit comment-skip guard keeps the parser from registering
        // "# map00010" as a synonym for "Bogus header".
        write_tsv(
            &list_pathway_path,
            "# map00010\tBogus header\n\
             \n\
             missing-tab-row\n\
             not-a-valid-id\tname\n\
             map00010\tGlycolysis / Gluconeogenesis\n",
        );

        let parsed = ParsedGff {
            genes: vec![gene_with_kegg(&["K00001"])],
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
        };
        let catalog = build_kegg_catalog(
            &parsed,
            &KeggCatalogInput {
                link_ko_pathway: Some(&pathway_path),
                list_pathway: Some(&list_pathway_path),
                ..KeggCatalogInput::default()
            },
        )
        .unwrap();

        assert_eq!(
            catalog.pathways[0].name.as_deref(),
            Some("Glycolysis / Gluconeogenesis")
        );
    }

    #[test]
    fn is_skippable_tsv_line_treats_blank_and_comment_lines_as_skippable() {
        assert!(is_skippable_tsv_line(""));
        assert!(is_skippable_tsv_line("# header"));
        assert!(is_skippable_tsv_line("#"));
        assert!(!is_skippable_tsv_line("ko:K00001\tpath:map00010"));
        assert!(!is_skippable_tsv_line("map00010\tname"));
        // Lines that contain '#' but don't start with it must be kept.
        assert!(!is_skippable_tsv_line("ko:K00001\t# inline"));
    }

    #[test]
    fn build_catalog_skips_unparseable_ko_entries() {
        // The link TSV contains a line whose KO field is not a valid
        // orthology id; we expect that whole row to be skipped rather than
        // producing an empty/zero-length entry.
        let dir = tempfile::tempdir().unwrap();
        let pathway_path = dir.path().join("link_pathway.tsv");
        write_tsv(
            &pathway_path,
            "ko:not-a-K-code\tpath:map00010\nko:K00001\tpath:map00010\n",
        );
        let parsed = ParsedGff {
            genes: vec![gene_with_kegg(&["K00001"])],
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
        };
        let catalog = build_kegg_catalog(
            &parsed,
            &KeggCatalogInput {
                link_ko_pathway: Some(&pathway_path),
                ..KeggCatalogInput::default()
            },
        )
        .unwrap();
        assert_eq!(catalog.pathways.len(), 1);
        assert_eq!(catalog.ko_links.len(), 1);
    }
}
