use anyhow::Result;
use log::info;

use crate::model::{AppConfig, OutputLayout};
use crate::{fs, manifest, ota, output};

pub fn run(config: &AppConfig, layout: &OutputLayout) -> Result<()> {
    info!(
        "Phase 1/3: extracting partition images from {}",
        config.input_zip.display()
    );
    let images = ota::extract_partition_images(&config.input_zip, &layout.images_dir)?;
    info!("Phase 1/3 complete: extracted {} image(s)", images.len());

    info!("Phase 2/3: extracting supported filesystem images");
    fs::extract_supported_images(&images, &layout.root)?;
    output::cleanup_images_dir(&layout.images_dir)?;

    info!("Phase 3/3: generating manifests");
    manifest::write_manifests(&layout.root, config.all_files_sha1)?;
    info!(
        "Pipeline complete: output available at {}",
        layout.root.display()
    );
    Ok(())
}
