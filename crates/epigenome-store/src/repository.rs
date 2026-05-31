use std::collections::HashMap;
use std::path::Path;

use epigenome_core::{
    EpigenomeRepository, Experiment, ExperimentId, ExperimentQuery, Peak, PeakHit, PeakRegionQuery,
};
use genome_core::AssemblyAccession;

use crate::dataset::EpigenomeDataset;
use crate::error::EpigenomeStoreError;
use crate::snapshot::read_snapshot;

/// In-memory `EpigenomeRepository` backed by an `EpigenomeDataset`.
///
/// Construction is O(experiments + peaks): per-experiment lookup tables and a
/// per-sequence peak index are built up front so per-query work stays bounded
/// by what's actually needed.
#[derive(Debug, Clone)]
pub struct FileEpigenomeRepository {
    dataset: EpigenomeDataset,
    experiment_by_id: HashMap<ExperimentId, Experiment>,
    peaks_by_experiment: HashMap<ExperimentId, Vec<Peak>>,
}

impl FileEpigenomeRepository {
    pub fn new(dataset: EpigenomeDataset) -> Result<Self, EpigenomeStoreError> {
        let mut experiment_by_id: HashMap<ExperimentId, Experiment> = HashMap::new();
        for experiment in &dataset.experiments {
            if experiment.assembly_accession != dataset.assembly_accession {
                return Err(EpigenomeStoreError::ExperimentAssemblyMismatch {
                    experiment: experiment.id.clone(),
                    experiment_assembly: experiment.assembly_accession.to_string(),
                    dataset_assembly: dataset.assembly_accession.to_string(),
                });
            }
            if experiment_by_id
                .insert(experiment.id.clone(), experiment.clone())
                .is_some()
            {
                return Err(EpigenomeStoreError::DuplicateExperiment(
                    experiment.id.clone(),
                ));
            }
        }

        let mut peaks_by_experiment: HashMap<ExperimentId, Vec<Peak>> = HashMap::new();
        for entry in &dataset.peaks {
            if !experiment_by_id.contains_key(&entry.experiment_id) {
                return Err(EpigenomeStoreError::UnknownExperimentInPeaks(
                    entry.experiment_id.clone(),
                ));
            }
            let mut peaks = entry.peaks.clone();
            peaks.sort_by(|a, b| {
                a.region
                    .sequence_name
                    .as_str()
                    .cmp(b.region.sequence_name.as_str())
                    .then_with(|| a.region.start.get().cmp(&b.region.start.get()))
            });
            peaks_by_experiment
                .entry(entry.experiment_id.clone())
                .or_default()
                .extend(peaks);
        }

        Ok(Self {
            dataset,
            experiment_by_id,
            peaks_by_experiment,
        })
    }

    pub fn from_snapshot_path(path: impl AsRef<Path>) -> Result<Self, EpigenomeStoreError> {
        let snapshot = read_snapshot(path)?;
        Self::new(snapshot.dataset)
    }

    pub fn assembly_accession(&self) -> &AssemblyAccession {
        &self.dataset.assembly_accession
    }

    pub fn experiment_count(&self) -> usize {
        self.experiment_by_id.len()
    }
}

impl EpigenomeRepository for FileEpigenomeRepository {
    fn experiment(&self, id: &ExperimentId) -> Option<Experiment> {
        self.experiment_by_id.get(id).cloned()
    }

    fn experiments(&self, query: &ExperimentQuery) -> Vec<Experiment> {
        let limit = query.limit.unwrap_or(usize::MAX);
        self.dataset
            .experiments
            .iter()
            .filter(|experiment| {
                query
                    .assembly_accession
                    .as_ref()
                    .is_none_or(|accession| &experiment.assembly_accession == accession)
                    && query.assay.is_none_or(|assay| experiment.assay == assay)
                    && query.target.as_ref().is_none_or(|target| {
                        experiment.target.as_ref().is_some_and(|t| t == target)
                    })
            })
            .take(limit)
            .cloned()
            .collect()
    }

    fn experiments_for_assembly(&self, accession: &AssemblyAccession) -> Vec<Experiment> {
        if &self.dataset.assembly_accession != accession {
            return Vec::new();
        }
        self.dataset.experiments.clone()
    }

    fn peaks_in_region(&self, query: &PeakRegionQuery) -> Vec<PeakHit> {
        if self.dataset.assembly_accession != query.assembly_accession {
            return Vec::new();
        }

        let limit = query.limit.unwrap_or(usize::MAX);
        let experiment_filter: Option<std::collections::HashSet<&ExperimentId>> =
            query.experiments.as_ref().map(|ids| ids.iter().collect());

        let mut hits = Vec::new();
        for experiment in &self.dataset.experiments {
            if let Some(filter) = &experiment_filter
                && !filter.contains(&experiment.id)
            {
                continue;
            }
            if query.assay.is_some_and(|assay| experiment.assay != assay) {
                continue;
            }
            if query
                .target
                .as_ref()
                .is_some_and(|target| experiment.target.as_ref() != Some(target))
            {
                continue;
            }

            let Some(peaks) = self.peaks_by_experiment.get(&experiment.id) else {
                continue;
            };
            for peak in peaks {
                if peak.region.overlaps(&query.region) {
                    hits.push(PeakHit {
                        experiment_id: experiment.id.clone(),
                        peak: peak.clone(),
                    });
                    if hits.len() >= limit {
                        return hits;
                    }
                }
            }
        }
        hits
    }

    fn peaks_for_experiment(&self, experiment_id: &ExperimentId) -> Vec<Peak> {
        self.peaks_by_experiment
            .get(experiment_id)
            .cloned()
            .unwrap_or_default()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use epigenome_core::{
        Antibody, Assay, ExperimentQc, ExperimentQuery, GeoSampleAccession, GeoSeriesAccession,
        PeakKind, Target,
    };
    use genome_core::{HalfOpenRegion, Position0, SequenceName, Strand};
    use std::collections::BTreeMap;

    use crate::dataset::ExperimentPeaks;

    fn assembly() -> AssemblyAccession {
        AssemblyAccession::new("GCA_037833805.1").unwrap()
    }

    fn region(sequence: &str, start: u64, end: u64) -> HalfOpenRegion {
        HalfOpenRegion::new(
            SequenceName::new(sequence).unwrap(),
            Position0::new(start),
            Position0::new(end),
        )
        .unwrap()
    }

    fn make_peak(sequence: &str, start: u64, end: u64, name: &str, summit: Option<u32>) -> Peak {
        Peak {
            region: region(sequence, start, end),
            name: name.to_owned(),
            score: 500,
            strand: Strand::Unknown,
            signal_value: 10.0,
            p_value: 20.0,
            q_value: 15.0,
            summit_offset: summit,
        }
    }

    fn experiment(id: &str, assay: Assay, target: Option<&str>) -> Experiment {
        Experiment {
            id: ExperimentId::new(id).unwrap(),
            assay,
            target: target.map(|t| Target::new(t).unwrap()),
            antibody: Some(Antibody::new("ab1").unwrap()),
            assembly_accession: assembly(),
            geo_series: Some(GeoSeriesAccession::new("GSE1").unwrap()),
            geo_sample: Some(GeoSampleAccession::new("GSM1").unwrap()),
            sra_runs: Vec::new(),
            tissue: Some("thallus".to_owned()),
            dev_stage: None,
            treatment: None,
            replicate: Some(1),
            pipeline: None,
            qvalue_cutoff: None,
            qc: ExperimentQc::default(),
            peak_kind: PeakKind::Narrow,
            signal_file: None,
            attributes: BTreeMap::new(),
        }
    }

    fn three_experiment_dataset() -> EpigenomeDataset {
        EpigenomeDataset {
            assembly_accession: assembly(),
            experiments: vec![
                experiment("h3k4me3_rep1", Assay::ChipSeq, Some("H3K4me3")),
                experiment("h3k27me3_rep1", Assay::ChipSeq, Some("H3K27me3")),
                experiment("atac_rep1", Assay::AtacSeq, None),
            ],
            peaks: vec![
                ExperimentPeaks {
                    experiment_id: ExperimentId::new("h3k4me3_rep1").unwrap(),
                    kind: PeakKind::Narrow,
                    peaks: vec![
                        make_peak("chr1", 100, 300, "h3k4_p1", Some(100)),
                        make_peak("chr1", 1000, 1500, "h3k4_p2", Some(250)),
                        make_peak("chr2", 50, 200, "h3k4_p3", Some(75)),
                    ],
                },
                ExperimentPeaks {
                    experiment_id: ExperimentId::new("h3k27me3_rep1").unwrap(),
                    kind: PeakKind::Broad,
                    peaks: vec![
                        make_peak("chr1", 500, 2000, "h3k27_p1", None),
                        make_peak("chr1", 5000, 8000, "h3k27_p2", None),
                    ],
                },
                ExperimentPeaks {
                    experiment_id: ExperimentId::new("atac_rep1").unwrap(),
                    kind: PeakKind::Narrow,
                    peaks: vec![
                        make_peak("chr1", 200, 400, "atac_p1", Some(50)),
                        make_peak("chr1", 2500, 2700, "atac_p2", Some(100)),
                    ],
                },
            ],
        }
    }

    #[test]
    fn construction_rejects_assembly_mismatch() {
        let mut dataset = three_experiment_dataset();
        dataset.experiments[0].assembly_accession = AssemblyAccession::new("GCA_other").unwrap();
        let err = FileEpigenomeRepository::new(dataset).unwrap_err();
        assert!(matches!(
            err,
            EpigenomeStoreError::ExperimentAssemblyMismatch { .. }
        ));
    }

    #[test]
    fn construction_rejects_duplicate_experiment_id() {
        let mut dataset = three_experiment_dataset();
        let dup = experiment("h3k4me3_rep1", Assay::AtacSeq, None);
        dataset.experiments.push(dup);
        let err = FileEpigenomeRepository::new(dataset).unwrap_err();
        assert!(matches!(err, EpigenomeStoreError::DuplicateExperiment(_)));
    }

    #[test]
    fn construction_rejects_peaks_for_unknown_experiment() {
        let mut dataset = three_experiment_dataset();
        dataset.peaks.push(ExperimentPeaks {
            experiment_id: ExperimentId::new("unknown_exp").unwrap(),
            kind: PeakKind::Narrow,
            peaks: vec![make_peak("chr1", 0, 100, "x", None)],
        });
        let err = FileEpigenomeRepository::new(dataset).unwrap_err();
        assert!(matches!(
            err,
            EpigenomeStoreError::UnknownExperimentInPeaks(_)
        ));
    }

    #[test]
    fn experiments_filters_by_assay_and_target() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let only_atac = repo.experiments(&ExperimentQuery {
            assay: Some(Assay::AtacSeq),
            ..Default::default()
        });
        assert_eq!(only_atac.len(), 1);
        assert_eq!(only_atac[0].id.as_str(), "atac_rep1");

        let h3k4 = repo.experiments(&ExperimentQuery {
            target: Some(Target::new("H3K4me3").unwrap()),
            ..Default::default()
        });
        assert_eq!(h3k4.len(), 1);
        assert_eq!(h3k4[0].id.as_str(), "h3k4me3_rep1");
    }

    #[test]
    fn experiments_for_assembly_returns_empty_for_other_assembly() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let other = AssemblyAccession::new("GCA_other").unwrap();
        assert!(repo.experiments_for_assembly(&other).is_empty());
        assert_eq!(repo.experiments_for_assembly(&assembly()).len(), 3);
    }

    #[test]
    fn peaks_in_region_returns_overlapping_peaks_only() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let hits = repo.peaks_in_region(&PeakRegionQuery {
            assembly_accession: assembly(),
            region: region("chr1", 0, 600),
            experiments: None,
            assay: None,
            target: None,
            limit: None,
        });

        // chr1 hits in [0..600): h3k4_p1 (100..300), h3k27_p1 (500..2000), atac_p1 (200..400)
        let names: Vec<_> = hits.iter().map(|h| h.peak.name.clone()).collect();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"h3k4_p1".to_owned()));
        assert!(names.contains(&"h3k27_p1".to_owned()));
        assert!(names.contains(&"atac_p1".to_owned()));
    }

    #[test]
    fn peaks_in_region_respects_assay_filter() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let hits = repo.peaks_in_region(&PeakRegionQuery {
            assembly_accession: assembly(),
            region: region("chr1", 0, 10000),
            experiments: None,
            assay: Some(Assay::AtacSeq),
            target: None,
            limit: None,
        });
        assert!(hits.iter().all(|h| h.experiment_id.as_str() == "atac_rep1"));
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn peaks_in_region_respects_experiment_whitelist() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let hits = repo.peaks_in_region(&PeakRegionQuery {
            assembly_accession: assembly(),
            region: region("chr1", 0, 10000),
            experiments: Some(vec![ExperimentId::new("h3k27me3_rep1").unwrap()]),
            assay: None,
            target: None,
            limit: None,
        });
        assert_eq!(hits.len(), 2);
        assert!(
            hits.iter()
                .all(|h| h.experiment_id.as_str() == "h3k27me3_rep1")
        );
    }

    #[test]
    fn peaks_in_region_returns_empty_for_other_assembly() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let hits = repo.peaks_in_region(&PeakRegionQuery {
            assembly_accession: AssemblyAccession::new("GCA_other").unwrap(),
            region: region("chr1", 0, 10000),
            experiments: None,
            assay: None,
            target: None,
            limit: None,
        });
        assert!(hits.is_empty());
    }

    #[test]
    fn peaks_in_region_honours_limit() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let hits = repo.peaks_in_region(&PeakRegionQuery {
            assembly_accession: assembly(),
            region: region("chr1", 0, 10000),
            experiments: None,
            assay: None,
            target: None,
            limit: Some(2),
        });
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn touching_regions_do_not_overlap() {
        // chr1 atac_p1 = [200..400). A region [100..200) touches but does not overlap.
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let hits = repo.peaks_in_region(&PeakRegionQuery {
            assembly_accession: assembly(),
            region: region("chr1", 100, 200),
            experiments: Some(vec![ExperimentId::new("atac_rep1").unwrap()]),
            assay: None,
            target: None,
            limit: None,
        });
        assert!(hits.is_empty());
    }

    #[test]
    fn peaks_for_experiment_returns_peaks_in_genomic_order() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let peaks = repo.peaks_for_experiment(&ExperimentId::new("h3k4me3_rep1").unwrap());
        assert_eq!(peaks.len(), 3);
        // Sorted by (sequence_name, start): chr1 100..300, chr1 1000..1500, chr2 50..200
        assert_eq!(peaks[0].name, "h3k4_p1");
        assert_eq!(peaks[1].name, "h3k4_p2");
        assert_eq!(peaks[2].name, "h3k4_p3");
    }

    #[test]
    fn peaks_for_unknown_experiment_returns_empty() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let peaks = repo.peaks_for_experiment(&ExperimentId::new("ghost").unwrap());
        assert!(peaks.is_empty());
    }

    #[test]
    fn experiments_filter_by_assembly_excludes_other_assemblies() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let matching = repo.experiments(&ExperimentQuery {
            assembly_accession: Some(assembly()),
            ..Default::default()
        });
        assert_eq!(matching.len(), 3);

        let other = repo.experiments(&ExperimentQuery {
            assembly_accession: Some(AssemblyAccession::new("GCA_other").unwrap()),
            ..Default::default()
        });
        assert!(other.is_empty());
    }

    #[test]
    fn peaks_in_region_excludes_experiments_with_different_target() {
        // Filtering by H3K4me3 must exclude H3K27me3 experiments — pins the
        // target inequality so mutating `!=` to `==` is caught.
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        let hits = repo.peaks_in_region(&PeakRegionQuery {
            assembly_accession: assembly(),
            region: region("chr1", 0, 10_000),
            experiments: None,
            assay: None,
            target: Some(Target::new("H3K4me3").unwrap()),
            limit: None,
        });
        assert!(!hits.is_empty());
        assert!(
            hits.iter()
                .all(|h| h.experiment_id.as_str() == "h3k4me3_rep1")
        );
    }

    #[test]
    fn experiment_count_matches_inserted_experiments() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        assert_eq!(repo.experiment_count(), 3);
    }

    #[test]
    fn experiment_returns_none_for_unknown_id() {
        let repo = FileEpigenomeRepository::new(three_experiment_dataset()).unwrap();
        assert!(
            repo.experiment(&ExperimentId::new("ghost").unwrap())
                .is_none()
        );
        assert!(
            repo.experiment(&ExperimentId::new("h3k4me3_rep1").unwrap())
                .is_some()
        );
    }
}
