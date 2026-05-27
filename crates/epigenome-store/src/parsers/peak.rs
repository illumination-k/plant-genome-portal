//! narrowPeak (BED6+4) and broadPeak (BED6+3) streaming parsers.
//!
//! UCSC format reference:
//!     <https://genome.ucsc.edu/FAQ/FAQformat.html#format12>
//!     <https://genome.ucsc.edu/FAQ/FAQformat.html#format13>
//!
//! Both formats use 0-based, half-open coordinates — the same as the portal's
//! internal `HalfOpenRegion`, so no offset conversion is needed.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use epigenome_core::Peak;
use flate2::read::MultiGzDecoder;
use genome_core::{HalfOpenRegion, Position0, SequenceName, Strand};

use crate::error::EpigenomeStoreError;

/// Open a peak file, transparently handling `.gz`.
pub fn open_peaks(path: impl AsRef<Path>) -> Result<Box<dyn BufRead>, EpigenomeStoreError> {
    let path = path.as_ref();
    let file = File::open(path)?;
    if path.extension().is_some_and(|ext| ext == "gz") {
        Ok(Box::new(BufReader::new(MultiGzDecoder::new(file))))
    } else {
        Ok(Box::new(BufReader::new(file)))
    }
}

/// Parse a narrowPeak (BED6+4) reader into a `Vec<Peak>`. The peak summit
/// column (10) populates `Peak::summit_offset`.
pub fn parse_narrow_peak(reader: impl Read) -> Result<Vec<Peak>, EpigenomeStoreError> {
    parse_peaks(reader, "narrowPeak", true)
}

/// Parse a broadPeak (BED6+3) reader into a `Vec<Peak>`. `Peak::summit_offset`
/// is always `None`.
pub fn parse_broad_peak(reader: impl Read) -> Result<Vec<Peak>, EpigenomeStoreError> {
    parse_peaks(reader, "broadPeak", false)
}

fn parse_peaks(
    reader: impl Read,
    format: &'static str,
    has_summit: bool,
) -> Result<Vec<Peak>, EpigenomeStoreError> {
    let reader = BufReader::new(reader);
    let mut peaks = Vec::new();
    let min_columns = if has_summit { 10 } else { 9 };

    for (line_index, line) in reader.lines().enumerate() {
        let line_number = line_index + 1;
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("track") {
            continue;
        }

        let cols: Vec<&str> = trimmed.split('\t').collect();
        if cols.len() < min_columns {
            return Err(EpigenomeStoreError::InvalidPeakLine {
                format,
                line: line_number,
                reason: format!(
                    "expected at least {} tab-separated columns, got {}",
                    min_columns,
                    cols.len()
                ),
            });
        }

        let chrom = parse_field(cols[0], format, line_number, "chrom", |v| {
            SequenceName::new(v).map_err(|err| err.to_string())
        })?;
        let start = parse_field(cols[1], format, line_number, "start", parse_u64)?;
        let end = parse_field(cols[2], format, line_number, "end", parse_u64)?;
        let name = cols[3].to_owned();
        let score = parse_field(cols[4], format, line_number, "score", parse_u16)?;
        let strand = parse_field(cols[5], format, line_number, "strand", |v| {
            v.parse::<Strand>().map_err(|err| err.to_string())
        })?;
        let signal_value = parse_field(cols[6], format, line_number, "signalValue", parse_f64)?;
        let p_value = parse_field(cols[7], format, line_number, "pValue", parse_f64)?;
        let q_value = parse_field(cols[8], format, line_number, "qValue", parse_f64)?;

        let summit_offset = if has_summit {
            parse_field(cols[9], format, line_number, "peak", parse_summit)?
        } else {
            None
        };

        let region = HalfOpenRegion::new(chrom, Position0::new(start), Position0::new(end))
            .map_err(|err| EpigenomeStoreError::InvalidPeakLine {
                format,
                line: line_number,
                reason: err.to_string(),
            })?;

        peaks.push(Peak {
            region,
            name,
            score,
            strand,
            signal_value,
            p_value,
            q_value,
            summit_offset,
        });
    }

    Ok(peaks)
}

fn parse_field<T, F>(
    value: &str,
    format: &'static str,
    line: usize,
    field: &'static str,
    convert: F,
) -> Result<T, EpigenomeStoreError>
where
    F: FnOnce(&str) -> Result<T, String>,
{
    convert(value).map_err(|err| EpigenomeStoreError::InvalidPeakLine {
        format,
        line,
        reason: format!("{field}: {err}"),
    })
}

fn parse_u64(value: &str) -> Result<u64, String> {
    value.parse::<u64>().map_err(|err| err.to_string())
}

fn parse_u16(value: &str) -> Result<u16, String> {
    let parsed: i64 = value
        .parse()
        .map_err(|err: std::num::ParseIntError| err.to_string())?;
    if !(0..=1000).contains(&parsed) {
        return Err(format!("score {parsed} out of range 0..=1000"));
    }
    Ok(parsed as u16)
}

fn parse_f64(value: &str) -> Result<f64, String> {
    value.parse::<f64>().map_err(|err| err.to_string())
}

fn parse_summit(value: &str) -> Result<Option<u32>, String> {
    let parsed: i64 = value
        .parse()
        .map_err(|err: std::num::ParseIntError| err.to_string())?;
    if parsed < 0 {
        // MACS writes -1 when no summit was called.
        return Ok(None);
    }
    u32::try_from(parsed)
        .map(Some)
        .map_err(|err| err.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    const NARROW_PEAK_SAMPLE: &str = "\
chr1\t100\t300\tpeak_1\t500\t.\t12.5\t30.0\t25.0\t75
chr1\t400\t800\tpeak_2\t1000\t+\t44.0\t-1\t100.0\t-1
# comment line
chr2\t50\t150\tpeak_3\t200\t-\t5.0\t10.0\t8.0\t40
";

    const BROAD_PEAK_SAMPLE: &str = "\
chr1\t1000\t5000\tbroad_1\t300\t.\t8.0\t20.0\t15.0
chr2\t2000\t9000\tbroad_2\t200\t.\t4.5\t12.0\t9.0
";

    #[test]
    fn narrow_peak_parses_three_rows() {
        let peaks = parse_narrow_peak(NARROW_PEAK_SAMPLE.as_bytes()).unwrap();
        assert_eq!(peaks.len(), 3);
        assert_eq!(peaks[0].name, "peak_1");
        assert_eq!(peaks[0].score, 500);
        assert_eq!(peaks[0].signal_value, 12.5);
        assert_eq!(peaks[0].summit_offset, Some(75));
        assert_eq!(peaks[0].strand, Strand::Unknown);
        assert_eq!(peaks[0].region.start.get(), 100);
        assert_eq!(peaks[0].region.end.get(), 300);

        // Missing summit (-1) → None.
        assert_eq!(peaks[1].summit_offset, None);
        assert_eq!(peaks[1].strand, Strand::Forward);
        assert_eq!(peaks[1].score, 1000);

        assert_eq!(peaks[2].name, "peak_3");
        assert_eq!(peaks[2].strand, Strand::Reverse);
    }

    #[test]
    fn broad_peak_parses_without_summit() {
        let peaks = parse_broad_peak(BROAD_PEAK_SAMPLE.as_bytes()).unwrap();
        assert_eq!(peaks.len(), 2);
        assert!(peaks.iter().all(|peak| peak.summit_offset.is_none()));
        assert_eq!(peaks[1].region.end.get(), 9000);
    }

    #[test]
    fn invalid_score_is_rejected() {
        let invalid = "chr1\t100\t200\tp\tNaN\t.\t1\t1\t1\t10\n";
        let err = parse_narrow_peak(invalid.as_bytes()).unwrap_err();
        match err {
            EpigenomeStoreError::InvalidPeakLine { format, line, .. } => {
                assert_eq!(format, "narrowPeak");
                assert_eq!(line, 1);
            }
            other => panic!("expected InvalidPeakLine, got {other:?}"),
        }
    }

    #[test]
    fn score_above_1000_is_rejected() {
        let invalid = "chr1\t100\t200\tp\t1500\t.\t1\t1\t1\t10\n";
        assert!(parse_narrow_peak(invalid.as_bytes()).is_err());
    }

    #[test]
    fn empty_lines_and_comments_are_skipped() {
        let input = "\n# header\nchr1\t0\t10\tp\t100\t.\t1\t1\t1\t5\n";
        let peaks = parse_narrow_peak(input.as_bytes()).unwrap();
        assert_eq!(peaks.len(), 1);
    }

    #[test]
    fn too_few_columns_is_rejected() {
        let input = "chr1\t0\t10\tp\t100\t.\t1\t1\n";
        let err = parse_narrow_peak(input.as_bytes()).unwrap_err();
        assert!(matches!(err, EpigenomeStoreError::InvalidPeakLine { .. }));
    }

    #[test]
    fn track_header_line_is_skipped() {
        let input = "track type=narrowPeak name=foo\nchr1\t0\t10\tp\t100\t.\t1\t1\t1\t5\n";
        let peaks = parse_narrow_peak(input.as_bytes()).unwrap();
        assert_eq!(peaks.len(), 1);
    }
}
