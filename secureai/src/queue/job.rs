use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum JobState {
    Pending,
    Running,
    Completed,
    Failed,
    TimedOut,
    Abandoned,
}

impl JobState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, JobState::Completed | JobState::Failed | JobState::Abandoned)
    }

    pub fn can_transition_to(&self, next: JobState) -> bool {
        match (self, next) {
            (JobState::Pending, JobState::Running) => true,
            (JobState::Running, JobState::Completed) => true,
            (JobState::Running, JobState::Failed) => true,
            (JobState::Running, JobState::TimedOut) => true,
            (JobState::Pending, JobState::Abandoned) => true,
            (JobState::Failed, JobState::Pending) => true, // Retry
            (JobState::TimedOut, JobState::Pending) => true, // Requeue
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub mime_type: String,
    pub data: Vec<u8>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub state: JobState,
    pub tool_name: String,
    pub tool_params: serde_json::Value,
    pub tenant_id: String,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub execution_log: String,
    pub artifacts: Vec<Artifact>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub lease_token: Option<String>,
    pub lease_expires_at: Option<u64>,
}

impl Job {
    pub fn new(
        tool_name: String,
        tool_params: serde_json::Value,
        tenant_id: String,
        max_retries: u32,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            state: JobState::Pending,
            tool_name,
            tool_params,
            tenant_id,
            created_at: now,
            started_at: None,
            completed_at: None,
            execution_log: String::new(),
            artifacts: Vec::new(),
            retry_count: 0,
            max_retries,
            lease_token: None,
            lease_expires_at: None,
        }
    }

    pub fn transition_to(&mut self, new_state: JobState) -> Result<(), JobError> {
        if !self.state.can_transition_to(new_state) {
            return Err(JobError::InvalidStateTransition {
                from: self.state,
                to: new_state,
            });
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        match new_state {
            JobState::Running => {
                self.started_at = Some(now);
            }
            JobState::Completed | JobState::Failed | JobState::TimedOut | JobState::Abandoned => {
                self.completed_at = Some(now);
                self.lease_token = None;
                self.lease_expires_at = None;
            }
            _ => {}
        }

        self.state = new_state;
        Ok(())
    }

    pub fn assign_lease(&mut self, token: String, expires_at: u64) {
        self.lease_token = Some(token);
        self.lease_expires_at = Some(expires_at);
    }

    pub fn clear_lease(&mut self) {
        self.lease_token = None;
        self.lease_expires_at = None;
    }

    pub fn has_active_lease(&self) -> bool {
        if let Some(expires_at) = self.lease_expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            expires_at > now
        } else {
            false
        }
    }

    pub fn append_log(&mut self, msg: &str) {
        if !self.execution_log.is_empty() {
            self.execution_log.push('\n');
        }
        self.execution_log.push_str(msg);
    }

    pub fn add_artifact(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lease {
    pub issued_at: u64,
    pub expires_at: u64,
}

impl Lease {
    pub fn new(duration_secs: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            issued_at: now,
            expires_at: now + (duration_secs * 1_000_000_000),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        now > self.expires_at
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum JobError {
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition { from: JobState, to: JobState },
}
