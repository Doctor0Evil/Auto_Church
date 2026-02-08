use crate::{job::{Job, JobKind}, queue::JobQueue, worker::Worker};

pub struct Scheduler {
    pub queue: JobQueue,
    pub worker: Worker,
}

impl Scheduler {
    pub fn new(worker_name: &str) -> Self {
        Self {
            queue: JobQueue::default(),
            worker: Worker::new(worker_name),
        }
    }

    pub fn enqueue_git_maintenance(&mut self, payload: serde_json::Value) {
        let job = Job::new(JobKind::GitMaintenance, payload);
        self.queue.push(job);
    }

    pub async fn run_once(&mut self) {
        if let Some(job) = self.queue.pop() {
            self.worker.execute(job).await;
        }
    }
}
