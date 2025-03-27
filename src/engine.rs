use crate::{
    db::{self, preg::dsl as preg_dsl, token::dsl as token_dsl},
    entry::{collect_entries_to_check, EntryStatus, ScanEntry},
    task::ScanTask,
};
use clap::ArgEnum;
use diesel::prelude::*;

#[allow(unused)]
#[derive(PartialEq, Eq, ArgEnum, Clone, Copy, Debug)]
pub enum ScanMod {
    Quick,
    Complete,
    Ai,
}

pub struct ScanEngine {
    scan_mod: ScanMod,
    tokens: Vec<db::Token>,
    pregs: Vec<db::Preg>,
}

impl Default for ScanEngine {
    fn default() -> Self {
        ScanEngine {
            scan_mod: ScanMod::Quick,
            tokens: Vec::new(),
            pregs: Vec::new(),
        }
    }
}

impl ScanEngine {
    pub fn new(scan_mod: ScanMod) -> Self {
        ScanEngine {
            scan_mod,
            tokens: Vec::new(),
            pregs: Vec::new(),
        }
    }

    pub fn run(&mut self, scan_path: std::path::PathBuf) -> Result<(), String> {
        // Create a ScanTask instance
        let mut task = ScanTask::new();

        // Load rules from database
        self.load_rules()
            .map_err(|e| format!("Failed to load rules: {}", e))?;

        // Initialize entries using scan_directory
        collect_entries_to_check(&scan_path, &mut task).map_err(|e| e.to_string())?;

        // Process entries by calling scan_file
        self.scan(&mut task);

        task.task_completed();

        Ok(())
    }

    pub fn load_rules(&mut self) -> Result<(), diesel::result::Error> {
        let mut conn = db::establish_connection();

        // Load tokens using Diesel's query DSL
        self.tokens = token_dsl::token.load::<db::Token>(&mut conn)?;

        // Load regex patterns using Diesel's query DSL
        self.pregs = preg_dsl::preg.load::<db::Preg>(&mut conn)?;

        log::info!(
            "Loaded {} tokens and {} regex patterns",
            self.tokens.len(),
            self.pregs.len()
        );

        Ok(())
    }

    fn scan(&self, task: &mut ScanTask) {
        match self.scan_mod {
            ScanMod::Quick => {
                for entry in &mut task.entries {
                    self.scan_file_quick(entry);
                }
            }
            ScanMod::Complete => {
                for entry in &mut task.entries {
                    self.scan_file_complete(entry);
                }
            }
            ScanMod::Ai => {
                for entry in &mut task.entries {
                    self.ai_scan_file(&entry.path);
                }
            }
        }
    }

    /// - Quick scan a file for tokens and regex patterns.
    /// - Once find a match then returns the warning level of the file.
    /// - Warning level is the sum of the levels of matched tokens and patterns.
    fn scan_file_quick(&self, entry: &mut ScanEntry) {
        // Open and read file
        match std::fs::File::open(&entry.path) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                if std::io::Read::read_to_end(&mut file, &mut buffer).is_err() {
                    log::error!("read file {:?} failed", entry.path);
                    entry.status = EntryStatus::Error;
                    return;
                }

                let scan_complete = |entry: &mut ScanEntry| {
                    entry.warning_level = entry.md5_matches + entry.preg_matches;
                    entry.status = if entry.warning_level > 0 {
                        EntryStatus::Danger
                    } else {
                        EntryStatus::Normal
                    };
                    log::info!(
                        "{} file {:?}: Warning level: {}, MD5 matches: {}, Preg matches: {}",
                        entry.status,
                        entry.path,
                        entry.warning_level,
                        entry.md5_matches,
                        entry.preg_matches,
                    );
                };

                // Check MD5 signatures (token-based scanning)
                for token in &self.tokens {
                    for i in 0..buffer.len().saturating_sub(token.len as usize) {
                        let chunk = &buffer[i..(i + token.len as usize)];
                        let result = format!("{:x}", md5::compute(chunk));

                        if result == token.token {
                            entry.md5_matches += std::cmp::max(token.level, 0) as usize;
                            scan_complete(entry);
                            return;
                        }
                    }
                }

                // Check regex patterns
                let content = String::from_utf8_lossy(&buffer);
                for preg in &self.pregs {
                    if let Ok(re) = regex::Regex::new(&preg.preg) {
                        let matches = re.find_iter(&content).count();
                        entry.preg_matches += matches * std::cmp::max(preg.level, 0) as usize;
                        if entry.preg_matches > 0 {
                            scan_complete(entry);
                            return;
                        }
                    }
                }
            }
            Err(_) => {
                log::error!("open file {:?} failed", entry.path);
                entry.status = EntryStatus::Error;
            }
        }
    }

    #[allow(unused)]
    fn ai_scan_file(&self, file_path: &std::path::Path) -> i32 {
        unimplemented!();
        // This would be the Rust equivalent of the AIScanFile function
        // It would execute a Python script and parse the output

        use std::process::Command;

        let path_str = file_path.to_string_lossy();

        let output = Command::new("python")
            .arg("check.py")
            .arg(path_str.as_ref())
            .output();

        // if let Ok(output) = output {
        //     let stdout = String::from_utf8_lossy(&output.stdout);
        //     if stdout.contains("\nW") {
        //         return -1; // Same as C++ code
        //     }
        // }
    }

    /// - Complete scan a file for tokens and regex patterns.
    /// - Returns the warning level of the file.
    /// - Warning level is the sum of the levels of matched tokens and patterns.
    fn scan_file_complete(&self, entry: &mut ScanEntry) {
        // Open and read file
        match std::fs::File::open(&entry.path) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                if std::io::Read::read_to_end(&mut file, &mut buffer).is_err() {
                    log::error!("read file {:?} failed", entry.path);
                    entry.status = EntryStatus::Error;
                    return;
                }

                // Check MD5 signatures (token-based scanning)
                for token in &self.tokens {
                    for i in 0..buffer.len().saturating_sub(token.len as usize) {
                        let chunk = &buffer[i..(i + token.len as usize)];
                        let result = format!("{:x}", md5::compute(chunk));

                        if result == token.token {
                            entry.md5_matches += std::cmp::max(token.level, 0) as usize;
                        }
                    }
                }

                // Check regex patterns
                let content = String::from_utf8_lossy(&buffer);
                for preg in &self.pregs {
                    if let Ok(re) = regex::Regex::new(&preg.preg) {
                        let matches = re.find_iter(&content).count();
                        entry.preg_matches += matches * std::cmp::max(preg.level, 0) as usize;
                        if entry.preg_matches > 0 {}
                    }
                }

                entry.warning_level = entry.md5_matches + entry.preg_matches;
                entry.status = if entry.warning_level > 0 {
                    EntryStatus::Danger
                } else {
                    EntryStatus::Normal
                };
                log::info!(
                    "{} file {:?}: Warning level: {}, MD5 matches: {}, Preg matches: {}",
                    entry.status,
                    entry.path,
                    entry.warning_level,
                    entry.md5_matches,
                    entry.preg_matches,
                );
            }
            Err(_) => {
                log::error!("open file {:?} failed", entry.path);
                entry.status = EntryStatus::Error;
            }
        }
    }
}

#[allow(unused)]
fn get_lines_from_buf(buffer: &[u8], pos: usize) -> u32 {
    buffer[..pos].iter().filter(|&&c| c == b'\n').count() as u32
}
