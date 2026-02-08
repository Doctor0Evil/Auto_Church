use crate::job::{Job, JobKind};

pub struct Worker {
    pub name: String,
}

impl Worker {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub async fn execute(&self, job: Job) {
        match job.kind {
            JobKind::GitMaintenance => {
                // bridge to ac_git_orchestrator
                println!("Worker {}: GitMaintenance {:?}", self.name, job.id.0);
            }
            JobKind::EcoScan => {
                println!("Worker {}: EcoScan {:?}", self.name, job.id.0);
            }
            JobKind::AuditLineage => {
                println!("Worker {}: AuditLineage {:?}", self.name, job.id.0);
            }
        }
    }
}
