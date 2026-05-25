use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::fmt;
use std::sync::{
    Arc, Mutex, MutexGuard,
    atomic::{AtomicU64, Ordering},
};
use std::thread;

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

pub trait JobExecutor<I, O>: Send + Sync + 'static {
    fn execute(&self, job: WorkerJob<I>) -> Result<O, String>;
}

impl<F, I, O, E> JobExecutor<I, O> for F
where
    F: Fn(WorkerJob<I>) -> Result<O, E> + Send + Sync + 'static,
    E: fmt::Display,
{
    fn execute(&self, job: WorkerJob<I>) -> Result<O, String> {
        self(job).map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct WorkerExecutor<W> {
    worker: W,
}

impl<W> WorkerExecutor<W> {
    pub fn new(worker: W) -> Self {
        Self { worker }
    }
}

impl<W> JobExecutor<W::Input, W::Output> for WorkerExecutor<W>
where
    W: Worker + Send + Sync + 'static,
{
    fn execute(&self, job: WorkerJob<W::Input>) -> Result<W::Output, String> {
        self.worker.run(job).map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

impl JobStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRecord<O> {
    pub id: String,
    pub kind: String,
    pub status: JobStatus,
    pub output: Option<O>,
    pub error: Option<String>,
}

impl<O> JobRecord<O> {
    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }
}

pub trait JobManager<I, O>: Send + Sync + 'static {
    fn submit(&self, kind: String, payload: I) -> Result<JobRecord<O>, JobManagerError>;
    fn get(&self, id: &str) -> Result<JobRecord<O>, JobManagerError>;
    fn list(&self) -> Vec<JobRecord<O>>;
}

pub struct InMemoryJobManager<I, O> {
    executor: Arc<dyn JobExecutor<I, O>>,
    jobs: Arc<Mutex<HashMap<String, JobRecord<O>>>>,
    next_id: Arc<AtomicU64>,
}

impl<I, O> InMemoryJobManager<I, O> {
    pub fn new<E>(executor: E) -> Self
    where
        E: JobExecutor<I, O>,
    {
        Self {
            executor: Arc::new(executor),
            jobs: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }
}

impl<I, O> Clone for InMemoryJobManager<I, O> {
    fn clone(&self) -> Self {
        Self {
            executor: Arc::clone(&self.executor),
            jobs: Arc::clone(&self.jobs),
            next_id: Arc::clone(&self.next_id),
        }
    }
}

impl<I, O> JobManager<I, O> for InMemoryJobManager<I, O>
where
    I: Send + 'static,
    O: Clone + Send + 'static,
{
    fn submit(&self, kind: String, payload: I) -> Result<JobRecord<O>, JobManagerError> {
        let sequence = self.next_id.fetch_add(1, Ordering::Relaxed);
        let id = format!("job-{sequence}");
        let record = JobRecord {
            id: id.clone(),
            kind: kind.clone(),
            status: JobStatus::Queued,
            output: None,
            error: None,
        };

        lock(&self.jobs).insert(id.clone(), record.clone());

        let executor = Arc::clone(&self.executor);
        let jobs = Arc::clone(&self.jobs);
        let thread_id = id.clone();
        let job = WorkerJob { id, kind, payload };

        thread::Builder::new()
            .name(format!("job-manager-{thread_id}"))
            .spawn(move || {
                update_job(&jobs, &thread_id, |record| {
                    record.status = JobStatus::Running;
                });

                match executor.execute(job) {
                    Ok(output) => update_job(&jobs, &thread_id, |record| {
                        record.status = JobStatus::Succeeded;
                        record.output = Some(output);
                        record.error = None;
                    }),
                    Err(error) => update_job(&jobs, &thread_id, |record| {
                        record.status = JobStatus::Failed;
                        record.output = None;
                        record.error = Some(error);
                    }),
                }
            })
            .map_err(|error| JobManagerError::SubmissionFailed(error.to_string()))?;

        Ok(record)
    }

    fn get(&self, id: &str) -> Result<JobRecord<O>, JobManagerError> {
        lock(&self.jobs)
            .get(id)
            .cloned()
            .ok_or_else(|| JobManagerError::JobNotFound(id.to_owned()))
    }

    fn list(&self) -> Vec<JobRecord<O>> {
        let mut records = lock(&self.jobs).values().cloned().collect::<Vec<_>>();
        records.sort_by(|left, right| left.id.cmp(&right.id));
        records
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JobManagerError {
    #[error("job not found: {0}")]
    JobNotFound(String),
    #[error("failed to submit job: {0}")]
    SubmissionFailed(String),
}

fn update_job<O>(
    jobs: &Mutex<HashMap<String, JobRecord<O>>>,
    id: &str,
    update: impl FnOnce(&mut JobRecord<O>),
) {
    if let Some(record) = lock(jobs).get_mut(id) {
        update(record);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn in_memory_job_manager_stores_successful_result() {
        let manager = InMemoryJobManager::new(|job: WorkerJob<u32>| -> Result<u32, String> {
            Ok(job.payload + 1)
        });

        let submitted = manager.submit("test.increment".to_owned(), 41).unwrap();
        let finished = wait_for_terminal(&manager, &submitted.id);

        assert_eq!(finished.status, JobStatus::Succeeded);
        assert_eq!(finished.output, Some(42));
        assert_eq!(finished.error, None);
    }

    #[test]
    fn in_memory_job_manager_stores_failed_result() {
        let manager = InMemoryJobManager::new(|_job: WorkerJob<()>| -> Result<(), String> {
            Err("worker failed".to_owned())
        });

        let submitted = manager.submit("test.fail".to_owned(), ()).unwrap();
        let finished = wait_for_terminal(&manager, &submitted.id);

        assert_eq!(finished.status, JobStatus::Failed);
        assert_eq!(finished.output, None);
        assert_eq!(finished.error, Some("worker failed".to_owned()));
    }

    #[test]
    fn worker_executor_adapts_existing_worker_trait() {
        let manager = InMemoryJobManager::new(WorkerExecutor::new(DoubleWorker));

        let submitted = manager.submit("test.double".to_owned(), 21).unwrap();
        let finished = wait_for_terminal(&manager, &submitted.id);

        assert_eq!(finished.status, JobStatus::Succeeded);
        assert_eq!(finished.output, Some(42));
    }

    fn wait_for_terminal<O: Clone + Send + 'static>(
        manager: &InMemoryJobManager<impl Send + 'static, O>,
        id: &str,
    ) -> JobRecord<O> {
        for _ in 0..100 {
            let record = manager.get(id).unwrap();
            if record.is_terminal() {
                return record;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("job did not finish");
    }

    #[derive(Debug, Clone)]
    struct DoubleWorker;

    impl Worker for DoubleWorker {
        type Input = u32;
        type Output = u32;
        type Error = TestError;

        fn run(&self, job: WorkerJob<Self::Input>) -> Result<Self::Output, Self::Error> {
            Ok(job.payload * 2)
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("test error")]
    struct TestError;
}
