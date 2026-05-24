use serde::{Serialize, de::DeserializeOwned};
use std::marker::PhantomData;
use thiserror::Error;

pub trait JobCodec<T> {
    type Error: std::error::Error + Send + Sync + 'static;

    fn encode(value: &T) -> Result<Vec<u8>, Self::Error>;
    fn decode(bytes: &[u8]) -> Result<T, Self::Error>;
}

pub struct MessagePack<T>(PhantomData<T>);

impl<T> JobCodec<T> for MessagePack<T>
where
    T: Serialize + DeserializeOwned,
{
    type Error = MessagePackError;

    fn encode(value: &T) -> Result<Vec<u8>, Self::Error> {
        rmp_serde::to_vec_named(value).map_err(MessagePackError::Encode)
    }

    fn decode(bytes: &[u8]) -> Result<T, Self::Error> {
        rmp_serde::from_slice(bytes).map_err(MessagePackError::Decode)
    }
}

#[derive(Debug, Error)]
pub enum MessagePackError {
    #[error(transparent)]
    Encode(#[from] rmp_serde::encode::Error),
    #[error(transparent)]
    Decode(#[from] rmp_serde::decode::Error),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use service::WorkerJob;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestPayload {
        value: String,
    }

    #[test]
    fn message_pack_roundtrips_typed_job() {
        let job = WorkerJob {
            id: "job-1".to_owned(),
            kind: "test".to_owned(),
            payload: TestPayload {
                value: "payload".to_owned(),
            },
        };

        let encoded = MessagePack::<WorkerJob<TestPayload>>::encode(&job).unwrap();
        let decoded = MessagePack::<WorkerJob<TestPayload>>::decode(&encoded).unwrap();

        assert_eq!(decoded, job);
    }
}
