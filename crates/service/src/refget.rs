use genome_core::GenomeRepository;

use crate::{GenomeService, ServiceError};

impl<R> GenomeService<R>
where
    R: GenomeRepository,
{
    pub fn refget_sequence(
        &self,
        checksum: &str,
        start: Option<u64>,
        end: Option<u64>,
    ) -> Result<String, ServiceError> {
        if self.repository.sequence_by_checksum(checksum).is_none() {
            return Err(ServiceError::SequenceNotFound(checksum.to_owned()));
        }

        let reference = self
            .reference
            .as_ref()
            .ok_or_else(|| ServiceError::SequenceNotFound(checksum.to_owned()))?;

        reference
            .get(checksum, start, end)
            .ok_or_else(|| ServiceError::InvalidRequest("invalid sequence range".to_owned()))
    }
}
