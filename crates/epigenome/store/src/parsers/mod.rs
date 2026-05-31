//! Parsers for MACS-style peak files (narrowPeak / broadPeak) and the
//! curator-facing TOML manifest that drives `portal-cli import
//! epigenome-manifest`.

mod manifest;
mod peak;

pub use manifest::{ExperimentManifestEntry, ManifestQc, parse_manifest};
pub use peak::{open_peaks, parse_broad_peak, parse_narrow_peak};
