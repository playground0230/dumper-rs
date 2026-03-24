use std::{env, ffi::OsStr, fs, path::Path, path::PathBuf};

use anyhow::{Context, Result, anyhow, ensure};
use log::{debug, info};

use crate::model::{AppConfig, OutputLayout};

pub fn prepare_output_layout(config: &AppConfig) -> Result<OutputLayout> {
    let root = resolve_output_root(&config.input_zip, config.output_dir_override.as_deref())?;
    let images_dir = root.join("images");

    info!("Using output root {}", root.display());
    debug!(
        "Ensuring images directory exists at {}",
        images_dir.display()
    );
    fs::create_dir_all(&images_dir)
        .with_context(|| format!("failed to create {}", images_dir.display()))?;

    Ok(OutputLayout { root, images_dir })
}

pub fn cleanup_images_dir(images_dir: &Path) -> Result<()> {
    if !images_dir.exists() {
        debug!(
            "Skipping image cleanup because {} does not exist",
            images_dir.display()
        );
        return Ok(());
    }

    info!(
        "Removing temporary image directory {}",
        images_dir.display()
    );
    fs::remove_dir_all(images_dir)
        .with_context(|| format!("failed to remove {}", images_dir.display()))?;
    Ok(())
}

pub fn resolve_output_root(
    input_zip: &Path,
    output_dir_override: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(path) = output_dir_override {
        debug!("Using output directory override {}", path.display());
        return Ok(path.to_path_buf());
    }

    let cwd = env::current_dir().context("failed to resolve current directory")?;
    let stem = input_zip
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| anyhow!("input path must have a valid UTF-8 filename stem"))?;

    ensure!(
        !stem.is_empty(),
        "input path must have a non-empty filename stem"
    );

    let resolved = cwd.join("output").join(stem);
    debug!(
        "Resolved default output directory for {} to {}",
        input_zip.display(),
        resolved.display()
    );
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::tempdir;

    use super::{cleanup_images_dir, resolve_output_root};

    #[test]
    fn resolves_default_output_from_input_stem() {
        let output = resolve_output_root(Path::new("/hdd/otas/12R_500.zip"), None).unwrap();
        let cwd = std::env::current_dir().unwrap();

        assert_eq!(output, cwd.join("output").join("12R_500"));
    }

    #[test]
    fn respects_output_override() {
        let output = resolve_output_root(
            Path::new("/hdd/otas/12R_500.zip"),
            Some(Path::new("/tmp/custom-output")),
        )
        .unwrap();

        assert_eq!(output, Path::new("/tmp/custom-output"));
    }

    #[test]
    fn removes_images_dir_tree() {
        let tempdir = tempdir().unwrap();
        let images_dir = tempdir.path().join("images");
        fs::create_dir_all(images_dir.join("nested")).unwrap();
        fs::write(images_dir.join("nested/system.img"), b"img").unwrap();

        cleanup_images_dir(&images_dir).unwrap();

        assert!(!images_dir.exists());
    }
}
