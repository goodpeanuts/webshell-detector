use chrono::Local;
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
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
    #[clap(arg_enum, short, long, default_value_t = ScanMod::Regex)]
    r#mod: ScanMod,
}

fn setup_logging(scan_mod: ScanMod) -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    // Create logs directory if it doesn't exist
    let logs_dir = PathBuf::from("logs");
    if !logs_dir.exists() {
        fs::create_dir_all(&logs_dir)?;
    }

    // Generate log file name with date and version
    let date = Local::now().format("%Y%m%d%H%M").to_string();
    let version = env!("CARGO_PKG_VERSION");
    let log_file = logs_dir.join(format!("{}_{}_{}.log", date, version, scan_mod));

    // Configure logging to write to both terminal and file
    let file_appender = tracing_appender::rolling::never(logs_dir, log_file.file_name().unwrap());
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    // Configure logging to write to both terminal and file
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout) // 输出到终端
        .with_ansi(true); // 保留终端颜色

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_file) // 输出到文件
        .with_ansi(false); // 禁用文件中的颜色

    tracing_subscriber::registry()
        .with(stdout_layer) // 添加终端输出
        .with(file_layer) // 添加文件输出
        .init();

    tracing::info!(
        "Logging initialized. Logs will be written to {:?}",
        log_file
    );

    Ok(guard)
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Parse command-line arguments
    let args = Cli::parse();
    let guard = setup_logging(args.r#mod)?;
    let scan_path = std::path::PathBuf::from(args.path);
    if !scan_path.exists() {
        anyhow::bail!("Error: {} does not exist", scan_path.display());
    }

    // Initialize and run the scan engine
    let mut engine = ScanEngine::new(args.r#mod);
    engine.run(scan_path).map_err(|e| anyhow::anyhow!(e))?;
    drop(guard);
    Ok(())
}
