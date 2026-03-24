mod app;
mod cli;
mod fs;
mod manifest;
mod model;
mod ota;
mod output;
mod pipeline;

use anyhow::Result;
use clap::Parser;
use log::{LevelFilter, debug};

use crate::cli::CliArgs;

fn main() -> Result<()> {
    init_logging();
    debug!("Logger initialized");
    app::run(CliArgs::parse())
}

fn init_logging() {
    let mut builder = pretty_env_logger::formatted_timed_builder();
    match std::env::var("RUST_LOG") {
        Ok(log_filter)
            if log_filter.contains("trace")
                || log_filter.contains("debug")
                || log_filter.contains("info") =>
        {
            builder.parse_filters(&log_filter);
        }
        _ => {
            builder.filter(None, LevelFilter::Info);
        }
    }
    builder.init();
}
