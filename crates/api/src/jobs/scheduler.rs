//! Job scheduler infrastructure for background tasks.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

/// Job frequency for scheduling.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Seconds and Daily are available for future jobs
pub enum JobFrequency {
    /// Run every N seconds (for testing).
    Seconds(u64),
    /// Run every N minutes.
    Minutes(u64),
    /// Run every hour.
    Hourly,
    /// Run every day at midnight UTC.
    Daily,
}

impl JobFrequency {
    /// Get the duration between job executions.
    pub fn duration(&self) -> Duration {
        match self {
            JobFrequency::Seconds(secs) => Duration::from_secs(*secs),
            JobFrequency::Minutes(mins) => Duration::from_secs(*mins * 60),
            JobFrequency::Hourly => Duration::from_secs(3600),
            JobFrequency::Daily => Duration::from_secs(86400),
        }
    }
}

/// Trait for implementing background jobs.
#[async_trait::async_trait]
pub trait Job: Send + Sync {
    /// The name of this job (used for logging).
    fn name(&self) -> &'static str;

    /// The frequency at which this job should run.
    fn frequency(&self) -> JobFrequency;

    /// Execute the job. Returns Ok(()) on success, Err with message on failure.
    async fn execute(&self) -> Result<(), String>;
}

/// Background job scheduler.
pub struct JobScheduler {
    jobs: Vec<Arc<dyn Job>>,
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
    handles: Vec<JoinHandle<()>>,
}

impl JobScheduler {
    /// Create a new job scheduler.
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        Self {
            jobs: Vec::new(),
            shutdown_tx,
            shutdown_rx,
            handles: Vec::new(),
        }
    }

    /// Register a job with the scheduler.
    pub fn register<J: Job + 'static>(&mut self, job: J) {
        self.jobs.push(Arc::new(job));
    }

    /// Start all registered jobs.
    pub fn start(&mut self) {
        info!("Starting job scheduler with {} jobs", self.jobs.len());

        for job in &self.jobs {
            let job = Arc::clone(job);
            let mut shutdown_rx = self.shutdown_rx.clone();

            let handle = tokio::spawn(async move {
                let name = job.name();
                let frequency = job.frequency();
                let mut interval = tokio::time::interval(frequency.duration());

                // Skip the first immediate tick
                interval.tick().await;

                info!(job = name, frequency = ?frequency, "Job scheduled");

                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            let start = std::time::Instant::now();
                            info!(job = name, "Job starting");

                            match job.execute().await {
                                Ok(()) => {
                                    let elapsed = start.elapsed();
                                    info!(
                                        job = name,
                                        elapsed_ms = elapsed.as_millis(),
                                        "Job completed successfully"
                                    );
                                }
                                Err(e) => {
                                    let elapsed = start.elapsed();
                                    error!(
                                        job = name,
                                        elapsed_ms = elapsed.as_millis(),
                                        error = %e,
                                        "Job failed"
                                    );
                                }
                            }
                        }
                        _ = shutdown_rx.changed() => {
                            if *shutdown_rx.borrow() {
                                info!(job = name, "Job shutting down");
                                break;
                            }
                        }
                    }
                }
            });

            self.handles.push(handle);
        }
    }

    /// Initiate graceful shutdown of all jobs.
    /// Returns immediately after signaling shutdown.
    pub fn shutdown(&self) {
        info!("Initiating job scheduler shutdown");
        let _ = self.shutdown_tx.send(true);
    }

    /// Wait for all jobs to complete with timeout.
    pub async fn wait_for_shutdown(self, timeout: Duration) {
        info!("Waiting for jobs to complete (timeout: {:?})", timeout);

        let shutdown_future = async {
            for handle in self.handles {
                if let Err(e) = handle.await {
                    warn!("Job task panicked: {}", e);
                }
            }
        };

        match tokio::time::timeout(timeout, shutdown_future).await {
            Ok(()) => info!("All jobs completed gracefully"),
            Err(_) => warn!("Job shutdown timed out after {:?}", timeout),
        }
    }
}

impl Default for JobScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestJob {
        run_count: Arc<AtomicUsize>,
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl Job for TestJob {
        fn name(&self) -> &'static str {
            "test_job"
        }

        fn frequency(&self) -> JobFrequency {
            JobFrequency::Seconds(1)
        }

        async fn execute(&self) -> Result<(), String> {
            self.run_count.fetch_add(1, Ordering::SeqCst);
            if self.should_fail {
                Err("Test failure".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_job_frequency_duration() {
        assert_eq!(
            JobFrequency::Seconds(30).duration(),
            Duration::from_secs(30)
        );
        assert_eq!(JobFrequency::Hourly.duration(), Duration::from_secs(3600));
        assert_eq!(JobFrequency::Daily.duration(), Duration::from_secs(86400));
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler = JobScheduler::new();
        assert!(scheduler.jobs.is_empty());
        assert!(scheduler.handles.is_empty());
    }

    #[test]
    fn test_scheduler_register() {
        let mut scheduler = JobScheduler::new();
        let job = TestJob {
            run_count: Arc::new(AtomicUsize::new(0)),
            should_fail: false,
        };
        scheduler.register(job);
        assert_eq!(scheduler.jobs.len(), 1);
    }

    #[tokio::test]
    async fn test_scheduler_shutdown() {
        let mut scheduler = JobScheduler::new();
        let run_count = Arc::new(AtomicUsize::new(0));
        let job = TestJob {
            run_count: Arc::clone(&run_count),
            should_fail: false,
        };
        scheduler.register(job);
        scheduler.start();

        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        scheduler.shutdown();
        scheduler.wait_for_shutdown(Duration::from_secs(2)).await;

        // Job should have been scheduled but not run yet (first tick skipped)
    }

    #[test]
    fn test_job_trait() {
        let job = TestJob {
            run_count: Arc::new(AtomicUsize::new(0)),
            should_fail: false,
        };
        assert_eq!(job.name(), "test_job");
        assert!(matches!(job.frequency(), JobFrequency::Seconds(1)));
    }

    #[test]
    fn test_scheduler_default() {
        let scheduler = JobScheduler::default();
        assert!(scheduler.jobs.is_empty());
    }
}
