use std::path::PathBuf;

use clap::{Parser, ValueHint};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Extract partition images, files, and manifests from Android OTA zips"
)]
pub struct CliArgs {
    /// Full OTA zip to process.
    #[arg(value_name = "INPUT_ZIP", value_hint = ValueHint::FilePath)]
    pub input_zip: PathBuf,

    /// Override the default output root.
    #[arg(long, value_name = "PATH", value_hint = ValueHint::DirPath)]
    pub output_dir: Option<PathBuf>,

    /// Also generate all_files.sha1sum.txt.
    #[arg(long)]
    pub all_files_sha1: bool,
}
