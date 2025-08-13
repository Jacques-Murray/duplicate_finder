use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The directory to scan for duplicate files
    #[arg(required = true, value_name = "DIRECTORY")]
    path: PathBuf,
}
