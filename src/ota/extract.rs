use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use log::{debug, info};
use otaripper::{ExtractOptions, OutputLocation};

use crate::model::PartitionImage;

pub fn extract_partition_images(
    input_zip: &Path,
    images_dir: &Path,
) -> Result<Vec<PartitionImage>> {
    info!(
        "Extracting partition images from {} with the otaripper library API",
        input_zip.display()
    );
    reset_directory(images_dir)?;

    let options = build_extract_options(images_dir);
    debug!("otaripper options: {:?}", options);
    let result = otaripper::extract(input_zip, options).with_context(|| {
        format!(
            "failed to extract partition images from {}",
            input_zip.display()
        )
    })?;

    info!(
        "otaripper wrote {} partition image(s) into {}",
        result.extracted_partitions.len(),
        result.output_dir.display()
    );

    let images = collect_partition_images(&result.output_dir)?;
    ensure!(
        !images.is_empty(),
        "otaripper did not produce any .img files in {}",
        result.output_dir.display()
    );

    info!(
        "Phase 1 complete: collected {} image(s) into {}",
        images.len(),
        result.output_dir.display()
    );
    Ok(images)
}

fn build_extract_options(images_dir: &Path) -> ExtractOptions {
    let mut options = ExtractOptions::new();
    options.output = OutputLocation::Exact(images_dir.to_path_buf());
    options
}

fn reset_directory(path: &Path) -> Result<()> {
    remove_existing_path(path)?;
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(())
}

fn remove_existing_path(path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to inspect {}", path.display()));
        }
    };

    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
    } else {
        fs::remove_file(path).with_context(|| format!("failed to remove {}", path.display()))?;
    }

    Ok(())
}

fn collect_partition_images(images_dir: &Path) -> Result<Vec<PartitionImage>> {
    let mut source_images = Vec::new();
    collect_image_files(images_dir, &mut source_images)?;
    source_images.sort();

    let mut images = Vec::with_capacity(source_images.len());
    for image_path in source_images {
        let partition_name = image_path
            .file_stem()
            .and_then(OsStr::to_str)
            .with_context(|| {
                format!(
                    "failed to derive partition name from {}",
                    image_path.display()
                )
            })?
            .to_owned();

        info!("Collected image {}", image_path.display());
        images.push(PartitionImage {
            name: partition_name,
            image_path,
        });
    }

    images.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(images)
}

fn collect_image_files(current_dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(current_dir)
        .with_context(|| format!("failed to read {}", current_dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("failed to enumerate {}", current_dir.display()))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?;
        let path = entry.path();

        if file_type.is_dir() {
            collect_image_files(&path, out)?;
            continue;
        }

        if file_type.is_file() && path.extension() == Some(OsStr::new("img")) {
            out.push(path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        path::Path,
    };

    use otaripper::OutputLocation;
    use tempfile::tempdir;

    use super::{build_extract_options, collect_partition_images};

    #[test]
    fn builds_extract_options_with_exact_output_location() {
        let options = build_extract_options(Path::new("/tmp/images"));
        assert_eq!(
            options.output,
            OutputLocation::Exact(Path::new("/tmp/images").into())
        );
        assert!(options.verify);
        assert!(!options.strict);
        assert!(!options.print_hash);
    }

    #[test]
    fn collects_partition_images_from_output_directory() {
        let tempdir = tempdir().unwrap();
        let images_dir = tempdir.path().join("images");
        fs::create_dir_all(images_dir.join("nested")).unwrap();

        File::create(images_dir.join("system.img")).unwrap();
        File::create(images_dir.join("nested/vendor.img")).unwrap();
        File::create(images_dir.join("payload.bin")).unwrap();

        let images = collect_partition_images(&images_dir).unwrap();
        let partitions = images
            .iter()
            .map(|image| image.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(partitions, vec!["system", "vendor"]);
        assert_eq!(images[0].image_path, images_dir.join("system.img"));
        assert_eq!(images[1].image_path, images_dir.join("nested/vendor.img"));
    }
}
