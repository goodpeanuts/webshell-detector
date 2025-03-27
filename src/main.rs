use webshell_detector::engine::ScanEngine;

fn main() {
    dotenvy::dotenv().ok();
    let mut engine = ScanEngine::new();
    env_logger::init();

    if let Err(e) = engine.run() {
        eprintln!("Error: {}", e);
    }
}
