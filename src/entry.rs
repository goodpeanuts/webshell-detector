use std::fmt;

use crate::task::ScanTask;

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
    pub path: std::path::PathBuf,
    pub md5_matches: usize,
    pub preg_matches: usize,
    pub warning_level: usize,
    pub status: EntryStatus,
}
pub fn collect_entries_to_check(
    dir_path: &std::path::Path,
    scan_task: &mut ScanTask,
) -> std::io::Result<()> {
    scan_directory(dir_path, scan_task)?;
    log::info!("[*] {} directories scanned", scan_task.dir_count);
    log::info!("====== Scanning completed ======");
    scan_task.refresh_status();
    Ok(())
}

/// - Scan a directory recursively for files with specific extensions.
fn scan_directory(dir_path: &std::path::Path, scan_task: &mut ScanTask) -> std::io::Result<()> {
    if !dir_path.is_dir() {
        log::error!("Directory not found: {:?}", dir_path);
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Directory not found",
        ));
    }

    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively scan subdirectory
            scan_task.dir_count += 1;
            scan_directory(&path, scan_task)?;
        } else if path.is_file() {
            // Check if file extension matches
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if scan_task
                    .extensions
                    .iter()
                    .any(|e| &ext_str == e || e == "*")
                {
                    // Add to entries with warning_level set to None
                    scan_task.entries.push(ScanEntry {
                        path: path.clone(),
                        md5_matches: 0,
                        preg_matches: 0,
                        warning_level: 0,
                        status: EntryStatus::Unchecked,
                    });
                }
            }
        }
    }

    Ok(())
}
