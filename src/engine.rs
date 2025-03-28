use crate::{
    db::{self, preg::dsl as preg_dsl, token::dsl as token_dsl},
    entry::ScanEntry,
    task::ScanTask,
    utils::scan_directory,
};
use clap::ArgEnum;
use diesel::prelude::*;
use std::fmt;

#[allow(unused)]
#[derive(PartialEq, Eq, ArgEnum, Clone, Copy, Debug)]
pub enum ScanMod {
    Regex,
    Hash,
    Complete,
    Ai,
}

impl fmt::Display for ScanMod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScanMod::Regex => write!(f, "Regex"),
            ScanMod::Hash => write!(f, "Hash"),
            ScanMod::Complete => write!(f, "Complete"),
            ScanMod::Ai => write!(f, "AI"),
        }
    }
}

pub struct ScanEngine {
    files: Vec<std::path::PathBuf>,
    scan_mod: ScanMod,
    tokens: Vec<db::Token>,
    pregs: Vec<db::Preg>,
}

impl Default for ScanEngine {
    fn default() -> Self {
        ScanEngine {
            files: Vec::new(),
            scan_mod: ScanMod::Regex,
            tokens: Vec::new(),
            pregs: Vec::new(),
        }
    }
}

impl ScanEngine {
    pub fn new(scan_mod: ScanMod) -> Self {
        ScanEngine {
            files: Vec::new(),
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

        self.files = scan_directory(&scan_path, &mut task)
            .map_err(|e| format!("Failed to scan directory: {}", e))?;

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

        tracing::info!(
            "Loaded {} tokens and {} regex patterns",
            self.tokens.len(),
            self.pregs.len()
        );

        Ok(())
    }

    fn scan(&self, task: &mut ScanTask) {
        match self.scan_mod {
            ScanMod::Regex => {
                task.collect_entries(self.scan_regex_quick(&self.files));
            }
            ScanMod::Hash => {
                task.collect_entries(self.scan_md5_quick(&self.files));
            }
            ScanMod::Complete => {
                task.collect_entries(self.scan_file_complete(&self.files));
            }
            ScanMod::Ai => {
                todo!()
            }
        }
    }

    /// - Quick scan a file for tokens and regex patterns.
    /// - Once find a match then returns the warning level of the file.
    /// - Warning level is the sum of the levels of matched tokens and patterns.
    fn scan_regex_quick(&self, files: &[std::path::PathBuf]) -> Vec<ScanEntry> {
        let mut scan_results = Vec::new();
        for file_path in files {
            if let Ok(mut file) = std::fs::File::open(file_path) {
                let mut buffer = Vec::new();
                if std::io::Read::read_to_end(&mut file, &mut buffer).is_err() {
                    tracing::error!("read file {:?} failed", file);
                    scan_results.push(ScanEntry::new_error(file_path.clone()));
                }

                // Check regex patterns
                let content = String::from_utf8_lossy(&buffer);
                for preg in &self.pregs {
                    if let Ok(re) = regex::Regex::new(&preg.preg) {
                        let matches = re.find_iter(&content).count();
                        let preg_matches = matches * std::cmp::max(preg.level, 0) as usize;
                        scan_results.push(ScanEntry::new_result_entry(
                            file_path.clone(),
                            0,
                            preg_matches,
                        ));
                        break;
                    }
                }

                if let Some(last_result) = scan_results.last() {
                    tracing::info!(
                        "{} file: {:?} Preg matches: {}",
                        last_result.status,
                        file_path,
                        last_result.preg_matches,
                    );
                } else {
                    tracing::error!("No scan results available for file: {:?}", file_path);
                }
            } else {
                tracing::error!("open file {:?} failed", file_path);
                scan_results.push(ScanEntry::new_error(file_path.clone()));
            }
        }
        scan_results
    }

    fn scan_md5_quick(&self, files: &[std::path::PathBuf]) -> Vec<ScanEntry> {
        let mut scan_results = Vec::new();
        for file_path in files {
            if let Ok(mut file) = std::fs::File::open(file_path) {
                let mut buffer = Vec::new();
                if std::io::Read::read_to_end(&mut file, &mut buffer).is_err() {
                    tracing::error!("read file {:?} failed", file);
                    scan_results.push(ScanEntry::new_error(file_path.clone()));
                }

                // Check MD5 signatures (token-based scanning)
                for token in &self.tokens {
                    for i in 0..buffer.len().saturating_sub(token.len as usize) {
                        let chunk = &buffer[i..(i + token.len as usize)];
                        let result = format!("{:x}", md5::compute(chunk));

                        if result == token.token {
                            let md5_matches = std::cmp::max(token.level, 0) as usize;
                            scan_results.push(ScanEntry::new_result_entry(
                                file_path.clone(),
                                md5_matches,
                                0,
                            ));
                            break;
                        }
                    }
                }

                if let Some(last_result) = scan_results.last() {
                    tracing::info!(
                        "{} file: {:?} MD5 matches: {}",
                        last_result.status,
                        file_path,
                        last_result.md5_matches,
                    );
                } else {
                    tracing::error!("No scan results available for file: {:?}", file);
                }
            } else {
                tracing::error!("open file {:?} failed", file_path);
                scan_results.push(ScanEntry::new_error(file_path.clone()));
            }
        }
        scan_results
    }

    #[allow(unused)]
    fn ai_scan_file(&self, files: &[std::path::PathBuf]) -> i32 {
        unimplemented!();
        //     // This would be the Rust equivalent of the AIScanFile function
        // It would execute a Python script and parse the output

        // use std::process::Command;

        // let path_str = file_path.to_string_lossy();

        // let output = Command::new("python")
        //     .arg("check.py")
        //     .arg(path_str.as_ref())
        //     .output();

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
    #[allow(unused)]
    fn scan_file_complete(&self, files: &[std::path::PathBuf]) -> Vec<ScanEntry> {
        let mut scan_results = Vec::new();
        for file_path in files {
            if let Ok(mut file) = std::fs::File::open(file_path) {
                let mut buffer = Vec::new();
                if std::io::Read::read_to_end(&mut file, &mut buffer).is_err() {
                    tracing::error!("read file {:?} failed", file);
                    scan_results.push(ScanEntry::new_error(file_path.clone()));
                }

                let mut md5_matches = 0;
                let mut preg_matches = 0;

                // Check MD5 signatures (token-based scanning)
                for token in &self.tokens {
                    for i in 0..buffer.len().saturating_sub(token.len as usize) {
                        let chunk = &buffer[i..(i + token.len as usize)];
                        let result = format!("{:x}", md5::compute(chunk));

                        if result == token.token {
                            md5_matches += std::cmp::max(token.level, 0) as usize;
                        }
                    }
                }

                // Check regex patterns
                let content = String::from_utf8_lossy(&buffer);
                for preg in &self.pregs {
                    if let Ok(re) = regex::Regex::new(&preg.preg) {
                        let matches = re.find_iter(&content).count();
                        preg_matches += matches * std::cmp::max(preg.level, 0) as usize;
                    }
                }

                scan_results.push(ScanEntry::new_result_entry(
                    file_path.clone(),
                    md5_matches,
                    preg_matches,
                ));

                if let Some(last_result) = scan_results.last() {
                    tracing::info!(
                        "{} file: {:?} MD5 matches: {} Preg matches: {}",
                        last_result.status,
                        file_path,
                        last_result.md5_matches,
                        last_result.preg_matches,
                    );
                } else {
                    tracing::error!("No scan results available for file: {:?}", file);
                }
            } else {
                tracing::error!("open file {:?} failed", file_path);
                scan_results.push(ScanEntry::new_error(file_path.clone()));
            }
        }
        scan_results
    }
}

#[allow(unused)]
fn get_lines_from_buf(buffer: &[u8], pos: usize) -> u32 {
    buffer[..pos].iter().filter(|&&c| c == b'\n').count() as u32
}
