use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Job {
    pub id: u32,
    pub pid: u32,
    pub command: String,
    pub status: JobStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Running,
    Done,
}

#[derive(Debug)]
pub struct JobManager {
    jobs: HashMap<u32, Job>,
    next_id: u32,
}

impl JobManager {
    pub fn new() -> Self {
        JobManager {
            jobs: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add(&mut self, pid: u32, command: String) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.jobs.insert(
            id,
            Job {
                id,
                pid,
                command,
                status: JobStatus::Running,
            },
        );
        id
    }

    /// Check if any jobs have finished (non-blocking).
    /// Returns the ids of jobs that changed status.
    pub fn reap_finished(&mut self) -> Vec<u32> {
        let mut reaped: Vec<u32> = Vec::new();
        let ids: Vec<u32> = self.jobs.keys().copied().collect();
        for id in ids {
            if let Some(job) = self.jobs.get(&id) {
                if job.status == JobStatus::Running {
                    // Check if process still exists via /proc
                    let alive: bool = std::path::Path::new(&format!("/proc/{}", job.pid)).exists();
                    if !alive {
                        if let Some(j) = self.jobs.get_mut(&id) {
                            j.status = JobStatus::Done;
                        }
                        reaped.push(id);
                    }
                }
            }
        }
        reaped
    }

    /// Get all jobs, sorted by id.
    pub fn all_sorted(&self) -> Vec<&Job> {
        let mut jobs: Vec<&Job> = self.jobs.values().collect();
        jobs.sort_by_key(|j| j.id);
        jobs
    }

    /// Get the most recent job (highest id).
    pub fn latest_id(&self) -> Option<u32> {
        self.jobs.keys().max().copied()
    }

    /// Get the second most recent job (second highest id).
    pub fn second_latest_id(&self) -> Option<u32> {
        let mut ids: Vec<&u32> = self.jobs.keys().collect();
        ids.sort();
        if ids.len() >= 2 {
            Some(**ids.get(ids.len() - 2).unwrap())
        } else {
            None
        }
    }

    /// Remove a job by id.
    pub fn remove(&mut self, id: u32) {
        self.jobs.remove(&id);
    }

    /// Remove all done jobs.
    pub fn remove_done(&mut self) {
        self.jobs.retain(|_, j| j.status == JobStatus::Running);
    }
}
