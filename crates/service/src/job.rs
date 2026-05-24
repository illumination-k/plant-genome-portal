use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerJob<I> {
    pub id: String,
    pub kind: String,
    pub payload: I,
}

pub trait Worker {
    type Input: Serialize + DeserializeOwned + Send + 'static;
    type Output: Serialize + DeserializeOwned + Send + 'static;
    type Error: std::error::Error + Send + Sync + 'static;

    fn run(&self, job: WorkerJob<Self::Input>) -> Result<Self::Output, Self::Error>;
}
