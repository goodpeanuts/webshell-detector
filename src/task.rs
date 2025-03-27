use std::fmt;

use crate::entry::{EntryStatus, ScanEntry};

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
    running_stutus: TaskStatus,
    pub entries: Vec<ScanEntry>,
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
            running_stutus: TaskStatus::Running,
            entries: Vec::new(),
            extensions: vec![
                "php".to_string(),
                "asp".to_string(),
                "aspx".to_string(),
                "jsp".to_string(),
                "html".to_string(),
            ],
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

    pub fn refresh_status(&mut self) {
        self.file_count = self.entries.len();
        self.error_count = self
            .entries
            .iter()
            .filter(|e| e.status == EntryStatus::Error)
            .count();
        self.danger_count = self
            .entries
            .iter()
            .filter(|e| e.status == EntryStatus::Danger)
            .count();
        if let Some(end_time) = self.end_time {
            self.duration = end_time.duration_since(self.start_time);
        } else {
            self.duration = self.start_time.elapsed();
        }

        let file_scaned = self
            .entries
            .iter()
            .filter(|e| e.status != EntryStatus::Unchecked)
            .count();

        // Log the updated status
        log::info!(
            "ScanTask Status {}: \n total file: {} and {file_scaned} files scanned \n Dangers: {}\n  Errors: {}, takes {:?}",
            self.running_stutus,
            self.file_count,
            self.error_count,
            self.danger_count,
            self.duration
        );
    }

    pub fn task_completed(&mut self) {
        if self.running_stutus == TaskStatus::Completed {
            return;
        }
        self.end_time = Some(std::time::Instant::now());
        self.running_stutus = TaskStatus::Completed;
        self.refresh_status();
    }
}
