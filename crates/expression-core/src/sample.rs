use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

use genome_core::AssemblyAccession;

use crate::ids::{
    BioProjectAccession, BioSampleAccession, SraExperimentAccession, SraRunAccession,
    SraStudyAccession,
};

/// Stable identity for one quantification unit in an RNA-seq dataset.
///
/// The primary key is the SRA Run (`SRR/ERR/DRR`), because one Run corresponds
/// to one FASTQ pair and therefore one expression quantification. Parent
/// accessions in the SRA hierarchy are carried alongside so callers can group
/// technical replicates by BioSample, or all samples in a study / project.
///
/// This type deliberately avoids organism-specific biological metadata such as
/// organ, tissue, developmental stage, genotype, or treatment. Those belong in
/// [`SampleMetadata::attributes`] and are interpreted by an organism-specific
/// [`SampleMetadataProfile`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SampleIdentity {
    pub run: SraRunAccession,
    pub experiment: Option<SraExperimentAccession>,
    pub study: Option<SraStudyAccession>,
    pub biosample: Option<BioSampleAccession>,
    pub bioproject: Option<BioProjectAccession>,
    pub assembly_accession: AssemblyAccession,
    pub title: Option<String>,
    pub description: Option<String>,
    pub library_strategy: Option<String>,
    pub library_layout: Option<String>,
    pub platform: Option<String>,
    pub instrument_model: Option<String>,
}

/// Organism-specific biological metadata for a sample.
///
/// The `attributes` map is the canonical extension point for values such as
/// `organ`, `tissue`, `developmental_stage`, `genotype`, `treatment`, or
/// `time_point`. Each organism can define a metadata profile that gives those
/// keys stable labels, grouping behavior, and sort order for visualization.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
pub struct SampleMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_key: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

/// One quantification unit in an RNA-seq dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Sample {
    pub identity: SampleIdentity,
    #[serde(default)]
    pub metadata: SampleMetadata,
}

impl Sample {
    pub fn run(&self) -> &SraRunAccession {
        &self.identity.run
    }

    pub fn study(&self) -> Option<&SraStudyAccession> {
        self.identity.study.as_ref()
    }

    pub fn biosample(&self) -> Option<&BioSampleAccession> {
        self.identity.biosample.as_ref()
    }

    pub fn bioproject(&self) -> Option<&BioProjectAccession> {
        self.identity.bioproject.as_ref()
    }

    pub fn assembly_accession(&self) -> &AssemblyAccession {
        &self.identity.assembly_accession
    }

    pub fn metadata_value(&self, key: &str) -> Option<&str> {
        self.metadata.attributes.get(key).map(String::as_str)
    }

    pub fn display_label(&self) -> String {
        self.metadata
            .display_label
            .clone()
            .or_else(|| self.identity.title.clone())
            .unwrap_or_else(|| self.identity.run.to_string())
    }
}

/// Defines how organism-specific sample metadata is exposed to visualization.
///
/// Implementations can live close to the organism importer. `expression-core`
/// only requires the behavior that cross-organism visualization needs:
/// labelling, grouping, faceting, and stable ordering.
pub trait SampleMetadataProfile: Send + Sync {
    fn profile_name(&self) -> &str;
    fn display_label(&self, sample: &Sample) -> String;
    fn primary_group_key(&self) -> Option<&str>;
    fn facet_keys(&self) -> &[String];
    fn stable_sort_key(&self, sample: &Sample) -> String;

    fn group_value(&self, sample: &Sample, key: &str) -> Option<String> {
        sample.metadata_value(key).map(ToOwned::to_owned)
    }
}

/// Metadata profile backed by keys in [`SampleMetadata::attributes`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeMetadataProfile {
    pub name: String,
    pub display_label_key: Option<String>,
    pub primary_group_key: Option<String>,
    pub facet_keys: Vec<String>,
    pub sort_key: Option<String>,
}

impl SampleMetadataProfile for AttributeMetadataProfile {
    fn profile_name(&self) -> &str {
        &self.name
    }

    fn display_label(&self, sample: &Sample) -> String {
        self.display_label_key
            .as_deref()
            .and_then(|key| sample.metadata_value(key))
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| sample.display_label())
    }

    fn primary_group_key(&self) -> Option<&str> {
        self.primary_group_key.as_deref()
    }

    fn facet_keys(&self) -> &[String] {
        &self.facet_keys
    }

    fn stable_sort_key(&self, sample: &Sample) -> String {
        self.sort_key
            .as_deref()
            .and_then(|key| sample.metadata_value(key))
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| sample.run().to_string())
    }
}

/// Top-level grouping aggregate corresponding to an INSDC BioProject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct BioProject {
    pub accession: BioProjectAccession,
    pub title: String,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn sample() -> Sample {
        let mut attributes = BTreeMap::new();
        attributes.insert("organ".to_owned(), "thallus".to_owned());
        attributes.insert("stage".to_owned(), "adult".to_owned());
        attributes.insert("sort_order".to_owned(), "002".to_owned());

        Sample {
            identity: SampleIdentity {
                run: SraRunAccession::new("SRR000001").unwrap(),
                experiment: None,
                study: None,
                biosample: None,
                bioproject: None,
                assembly_accession: AssemblyAccession::new("GCA_037833805.1").unwrap(),
                title: Some("Fallback title".to_owned()),
                description: None,
                library_strategy: Some("RNA-Seq".to_owned()),
                library_layout: None,
                platform: None,
                instrument_model: None,
            },
            metadata: SampleMetadata {
                profile: Some("marchantia_expression".to_owned()),
                display_label: None,
                primary_group: Some("organ".to_owned()),
                sort_key: Some("sort_order".to_owned()),
                attributes,
            },
        }
    }

    #[test]
    fn attribute_profile_reads_organism_specific_metadata() {
        let profile = AttributeMetadataProfile {
            name: "marchantia_expression".to_owned(),
            display_label_key: Some("organ".to_owned()),
            primary_group_key: Some("organ".to_owned()),
            facet_keys: vec!["organ".to_owned(), "stage".to_owned()],
            sort_key: Some("sort_order".to_owned()),
        };
        let sample = sample();

        assert_eq!(profile.profile_name(), "marchantia_expression");
        assert_eq!(profile.display_label(&sample), "thallus");
        assert_eq!(profile.primary_group_key(), Some("organ"));
        assert_eq!(profile.facet_keys(), ["organ", "stage"]);
        assert_eq!(
            profile.group_value(&sample, "stage").as_deref(),
            Some("adult")
        );
        assert_eq!(profile.stable_sort_key(&sample), "002");
    }

    #[test]
    fn sample_display_label_falls_back_to_title_then_run() {
        let mut sample = sample();
        assert_eq!(sample.display_label(), "Fallback title");

        sample.metadata.display_label = Some("Curated label".to_owned());
        assert_eq!(sample.display_label(), "Curated label");

        sample.metadata.display_label = None;
        sample.identity.title = None;
        assert_eq!(sample.display_label(), "SRR000001");
    }
}
