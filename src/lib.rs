use engine::ScanEngine;

pub mod db;
pub mod engine;
pub mod entry;
pub mod task;

pub fn run() {
    let mut engine = ScanEngine::new();

    if let Err(e) = engine.run() {
        eprintln!("Error: {}", e);
    }
}
