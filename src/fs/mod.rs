mod detect;
mod erofs;
mod ext4;

use std::{fs, path::Path};

use anyhow::{Context, Result};
use log::{debug, info, warn};

use crate::model::{FilesystemKind, PartitionImage, should_dump_partition_files};

pub use detect::detect_filesystem_kind;

pub fn extract_supported_images(images: &[PartitionImage], output_root: &Path) -> Result<()> {
    info!(
        "Preparing to extract {} filesystem image(s) into {}",
        images.len(),
        output_root.display()
    );
    for image in images {
        if !should_dump_partition_files(&image.name) {
            debug!(
                "Skipping filesystem dump for non-allowlisted partition {}",
                image.name
            );
            continue;
        }

        let Some(kind) = detect_filesystem_kind(&image.image_path)? else {
            warn!("Skipping unsupported image {}", image.image_path.display());
            continue;
        };

        let destination = output_root.join(&image.name);
        info!(
            "Extracting {} as {:?} into {}",
            image.image_path.display(),
            kind,
            destination.display()
        );
        prepare_partition_dir(&destination)?;

        match kind {
            FilesystemKind::Ext4 => ext4::extract_image(&image.image_path, &destination)?,
            FilesystemKind::Erofs => erofs::extract_image(&image.image_path, &destination)?,
        }
    }

    Ok(())
}

fn prepare_partition_dir(path: &Path) -> Result<()> {
    debug!("Preparing partition output directory {}", path.display());
    remove_existing_path(path)?;
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(())
}

fn remove_existing_path(path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to inspect {}", path.display()));
        }
    };

    debug!("Removing existing path {}", path.display());
    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
    } else {
        fs::remove_file(path).with_context(|| format!("failed to remove {}", path.display()))?;
    }

    Ok(())
}
