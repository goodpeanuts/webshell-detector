use engine::ScanEngine;
use std::path::PathBuf;

pub mod db;
pub mod engine;

pub fn run() {
    let mut engine = ScanEngine::new();

    // Load rules from database
    if let Err(e) = engine.load_rules() {
        eprintln!("Failed to load rules: {}", e);
        return;
    }

    println!(
        "Loaded {} token rules and {} pattern rules",
        engine.tokens.len(),
        engine.pregs.len()
    );

    // Scan directory with specific extensions
    let scan_dir = PathBuf::from("./dataset");
    let extensions = ["php", "asp", "aspx", "jsp", "html"];

    if let Err(e) = engine.scan_directory(&scan_dir, extensions.to_vec().as_ref()) {
        eprintln!("Scan error: {}", e);
    }

    println!("Scan completed.");
    println!("Files scanned: {}", engine.file_count);
    println!("Directories scanned: {}", engine.dir_count);
    println!("Errors encountered: {}", engine.error_count);
}
