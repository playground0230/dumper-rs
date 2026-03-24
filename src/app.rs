use anyhow::Result;
use log::{debug, info};

use crate::cli::CliArgs;
use crate::model::AppConfig;
use crate::{output, pipeline};

pub fn run(args: CliArgs) -> Result<()> {
    debug!("CLI args: {:?}", args);
    let config = AppConfig::from(args);
    info!("Starting dumper-rs");
    debug!("Resolved app config: {:?}", config);
    let layout = output::prepare_output_layout(&config)?;
    pipeline::run(&config, &layout)
}
