use crate::db::*;
use crate::schema::{preg::dsl as preg_dsl, token::dsl as token_dsl};
use diesel::{prelude::*, sqlite::SqliteConnection};
use dotenvy::dotenv;
use md5::compute;
use regex::Regex;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

pub struct ScanEngine {
    pub(crate) tokens: Vec<Token>,
    pub(crate) pregs: Vec<Preg>,
    pub(crate) running: bool,
    pub(crate) file_count: u64,
    pub(crate) dir_count: u64,
    pub(crate) error_count: u64,
}

impl Default for ScanEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScanEngine {
    pub fn new() -> Self {
        ScanEngine {
            tokens: Vec::new(),
            pregs: Vec::new(),
            running: true,
            file_count: 0,
            dir_count: 0,
            error_count: 0,
        }
    }

    fn establish_connection(&self) -> SqliteConnection {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        assert!(Path::new(&database_url).exists(), "Database file not found");
        SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
    }

    pub fn load_rules(&mut self) -> Result<(), diesel::result::Error> {
        let mut conn = self.establish_connection();

        // Load tokens using Diesel's query DSL
        self.tokens = token_dsl::token.load::<Token>(&mut conn)?;

        // Load regex patterns using Diesel's query DSL
        self.pregs = preg_dsl::preg.load::<Preg>(&mut conn)?;

        Ok(())
    }

    pub(crate) fn scan_directory(
        &mut self,
        dir_path: &Path,
        extensions: &[&str],
    ) -> io::Result<()> {
        if !dir_path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Directory not found",
            ));
        }

        self.dir_count += 1;

        for entry in fs::read_dir(dir_path)? {
            if !self.running {
                break;
            }

            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectory
                self.scan_directory(&path, extensions)?;
            } else if path.is_file() {
                // Check if file extension matches
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if extensions.iter().any(|&e| ext_str == e || e == "*") {
                        // Scan the file
                        let warning_level = self.scan_file(&path);
                        if warning_level > 0 {
                            println!("Warning level {} in file: {:?}", warning_level, path);
                        }
                        self.file_count += 1;
                    }
                }
            }
        }

        Ok(())
    }

    fn scan_file(&mut self, file_path: &Path) -> i32 {
        let mut warning_level = 0;

        // Open and read file
        match File::open(file_path) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                if file.read_to_end(&mut buffer).is_err() {
                    self.error_count += 1;
                    return 0;
                }

                // Check MD5 signatures (token-based scanning)
                for token in &self.tokens {
                    for i in 0..buffer.len().saturating_sub(token.len as usize) {
                        if !self.running {
                            break;
                        }

                        // Calculate MD5 of the chunk
                        let chunk = &buffer[i..(i + token.len as usize)];
                        let result = format!("{:x}", compute(chunk));

                        // Compare MD5 hex strings
                        if result == token.token {
                            warning_level += token.level;
                        }
                    }
                }

                // Check regex patterns
                let content = String::from_utf8_lossy(&buffer);
                for preg in &self.pregs {
                    if let Ok(re) = Regex::new(&preg.preg) {
                        let matches = re.find_iter(&content).count();
                        warning_level += (matches as i32) * preg.level;
                    }
                }

                // AI scan could be implemented here
                // if warning_level == 0 {
                //     warning_level = self.ai_scan_file(file_path);
                // }
            }
            Err(_) => {
                self.error_count += 1;
            }
        }

        warning_level
    }

    #[allow(unused)]
    fn ai_scan_file(&self, file_path: &Path) -> i32 {
        // This would be the Rust equivalent of the AIScanFile function
        // It would execute a Python script and parse the output

        use std::process::Command;

        let path_str = file_path.to_string_lossy();

        let output = Command::new("python")
            .arg("check.py")
            .arg(path_str.as_ref())
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("\nW") {
                return -1; // Same as C++ code
            }
        }

        0
    }
}

#[allow(unused)]
fn get_lines_from_buf(buffer: &[u8], pos: usize) -> u32 {
    buffer[..pos].iter().filter(|&&c| c == b'\n').count() as u32
}
