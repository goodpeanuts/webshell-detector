use std::fmt;

#[derive(PartialEq)]
pub enum EntryStatus {
    Unchecked,
    Normal,
    Danger,
    Error,
}

impl fmt::Display for EntryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            EntryStatus::Unchecked => "Unchecked",
            EntryStatus::Normal => "Normal",
            EntryStatus::Danger => "Danger",
            EntryStatus::Error => "Error",
        };
        write!(f, "{}", status_str)
    }
}

pub struct ScanEntry {
    pub file: std::path::PathBuf,
    pub md5_matches: usize,
    pub preg_matches: usize,
    pub warning_level: usize,
    pub status: EntryStatus,
}

impl ScanEntry {
    pub fn new(file: std::path::PathBuf) -> Self {
        ScanEntry {
            file,
            md5_matches: 0,
            preg_matches: 0,
            warning_level: 0,
            status: EntryStatus::Unchecked,
        }
    }

    pub fn new_error(file: std::path::PathBuf) -> Self {
        ScanEntry {
            file,
            md5_matches: 0,
            preg_matches: 0,
            warning_level: 0,
            status: EntryStatus::Error,
        }
    }

    pub fn new_result_entry(
        file: std::path::PathBuf,
        md5_matches: usize,
        preg_matches: usize,
    ) -> Self {
        let status = if md5_matches > 0 || preg_matches > 0 {
            EntryStatus::Danger
        } else {
            EntryStatus::Normal
        };

        ScanEntry {
            file,
            md5_matches,
            preg_matches,
            warning_level: md5_matches + preg_matches,
            status,
        }
    }
}
