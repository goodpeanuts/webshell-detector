use clap::Parser;
use webshell_detector::engine::{ScanEngine, ScanMod};

#[derive(Parser)]
#[clap(
    name = "Webshell Detector",
    version,
    about = "Detects webshells in specified directories"
)]
struct Cli {
    /// The directory to scan
    #[clap(short, long, default_value = "./dataset")]
    path: String,

    /// The scan mode: Quick, Complete, or Ai
    #[clap(arg_enum, short, long, default_value_t = ScanMod::Quick)]
    r#mod: ScanMod,
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    // Parse command-line arguments
    let args = Cli::parse();
    let scan_path = std::path::PathBuf::from(args.path);
    if !scan_path.exists() {
        anyhow::bail!("Error: {} does not exist", scan_path.display());
    }

    // Initialize and run the scan engine
    let mut engine = ScanEngine::new(args.r#mod);
    engine.run(scan_path).map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}
