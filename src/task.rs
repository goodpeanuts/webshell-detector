use crate::entry::{EntryStatus, ScanEntry};
use std::fmt;

#[allow(unused)]
#[derive(PartialEq, Eq)]
pub enum TaskStatus {
    Running,
    Paused,
    Break,
    Completed,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            TaskStatus::Running => "Running",
            TaskStatus::Paused => "Paused",
            TaskStatus::Break => "Break",
            TaskStatus::Completed => "Completed",
        };
        write!(f, "{}", status_str)
    }
}

pub struct ScanTask {
    running_status: TaskStatus,
    entries: Vec<ScanEntry>,
    pub extensions: Vec<String>,
    file_count: usize,
    pub dir_count: usize,
    error_count: usize,
    danger_count: usize,
    start_time: std::time::Instant,
    end_time: Option<std::time::Instant>,
    duration: std::time::Duration,
}

impl Default for ScanTask {
    fn default() -> Self {
        ScanTask {
            running_status: TaskStatus::Running,
            entries: Default::default(),
            extensions: vec![],
            file_count: 0,
            dir_count: 0,
            error_count: 0,
            danger_count: 0,
            start_time: std::time::Instant::now(),
            end_time: None,
            duration: std::time::Duration::new(0, 0),
        }
    }
}

impl ScanTask {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collect_entries(&mut self, result: Vec<ScanEntry>) {
        tracing::info!("Collected {} entries", result.len());
        self.entries.extend(result);
        self.refresh_status();
    }

    pub fn refresh_status(&mut self) {
        self.file_count = self.entries.len();
        self.error_count = self
            .entries
            .iter()
            .filter(|entry| entry.status == EntryStatus::Error)
            .count();
        self.danger_count = self
            .entries
            .iter()
            .filter(|entry| entry.status == EntryStatus::Danger)
            .count();
        if let Some(end_time) = self.end_time {
            self.duration = end_time.duration_since(self.start_time);
        } else {
            self.duration = self.start_time.elapsed();
        }

        let files_scanned = self
            .entries
            .iter()
            .filter(|entry| entry.status != EntryStatus::Unchecked)
            .count();

        // Log the updated status
        tracing::info!(
            "ScanTask Status {} \nFiles scanned: {files_scanned}/{}  \nDangers: {}\nErrors: {}\nTakes {:?}",
            self.running_status,
            self.file_count,
            self.danger_count,
            self.error_count,
            self.duration
        );
    }

    pub fn task_completed(&mut self) {
        if self.running_status == TaskStatus::Completed {
            return;
        }
        self.end_time = Some(std::time::Instant::now());
        self.running_status = TaskStatus::Completed;
        self.refresh_status();
    }
}
