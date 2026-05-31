use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

use crate::error::DomainError;
use crate::ids::SequenceName;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
pub struct Position0(u64);

impl Position0 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
pub struct Position1(u64);

impl Position1 {
    pub fn new(value: u64) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::ZeroPosition1);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn to_position0(self) -> Position0 {
        Position0(self.0 - 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ClosedRegion {
    pub sequence_name: SequenceName,
    pub start: Position1,
    pub end: Position1,
}

impl ClosedRegion {
    pub fn new(
        sequence_name: SequenceName,
        start: Position1,
        end: Position1,
    ) -> Result<Self, DomainError> {
        if start.get() > end.get() {
            return Err(DomainError::InvalidClosedRegion);
        }
        Ok(Self {
            sequence_name,
            start,
            end,
        })
    }

    pub fn to_half_open(&self) -> Result<HalfOpenRegion, DomainError> {
        HalfOpenRegion::new(
            self.sequence_name.clone(),
            self.start.to_position0(),
            Position0::new(self.end.get()),
        )
    }
}

impl FromStr for ClosedRegion {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some((sequence_name, range)) = value.split_once(':') else {
            return Err(DomainError::InvalidRegionExpression(value.to_owned()));
        };
        let Some((start, end)) = range.split_once('-') else {
            return Err(DomainError::InvalidRegionExpression(value.to_owned()));
        };

        let start = start
            .replace(',', "")
            .parse::<u64>()
            .map_err(|_| DomainError::InvalidRegionExpression(value.to_owned()))?;
        let end = end
            .replace(',', "")
            .parse::<u64>()
            .map_err(|_| DomainError::InvalidRegionExpression(value.to_owned()))?;

        Self::new(
            SequenceName::new(sequence_name)?,
            Position1::new(start)?,
            Position1::new(end)?,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct HalfOpenRegion {
    pub sequence_name: SequenceName,
    pub start: Position0,
    pub end: Position0,
}

impl HalfOpenRegion {
    pub fn new(
        sequence_name: SequenceName,
        start: Position0,
        end: Position0,
    ) -> Result<Self, DomainError> {
        if start.get() >= end.get() {
            return Err(DomainError::InvalidHalfOpenRegion);
        }
        Ok(Self {
            sequence_name,
            start,
            end,
        })
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.sequence_name == other.sequence_name
            && self.start.get() < other.end.get()
            && other.start.get() < self.end.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Strand {
    Forward,
    Reverse,
    Unknown,
}

impl FromStr for Strand {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "+" => Ok(Self::Forward),
            "-" => Ok(Self::Reverse),
            "." | "?" => Ok(Self::Unknown),
            other => Err(DomainError::InvalidStrand(other.to_owned())),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn closed_region_converts_to_half_open() {
        let region = ClosedRegion::from_str("chr1:1-10").unwrap();
        let half_open = region.to_half_open().unwrap();

        assert_eq!(half_open.start.get(), 0);
        assert_eq!(half_open.end.get(), 10);
    }

    #[test]
    fn half_open_overlap_uses_sequence_name() {
        let a = HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(0),
            Position0::new(10),
        )
        .unwrap();
        let b = HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(9),
            Position0::new(20),
        )
        .unwrap();
        let c = HalfOpenRegion::new(
            SequenceName::new("chr2").unwrap(),
            Position0::new(9),
            Position0::new(20),
        )
        .unwrap();

        assert!(a.overlaps(&b));
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn touching_half_open_regions_do_not_overlap() {
        let a = HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(0),
            Position0::new(10),
        )
        .unwrap();
        let b = HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(10),
            Position0::new(20),
        )
        .unwrap();

        assert!(!a.overlaps(&b));
        assert!(!b.overlaps(&a));
    }
}
