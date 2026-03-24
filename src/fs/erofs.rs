use std::{
    ffi::OsString,
    io,
    path::Path,
    process::{Command, ExitStatus},
};

use anyhow::{Result, anyhow, ensure};
use log::{debug, info};

const FSCK_EROFS_BINARY: &str = "fsck.erofs";

pub fn extract_image(image_path: &Path, destination: &Path) -> Result<()> {
    info!(
        "Extracting EROFS image {} into {} with fsck.erofs",
        image_path.display(),
        destination.display()
    );

    ensure_fsck_erofs_available()?;
    run_fsck_erofs_extract(image_path, destination)?;

    info!(
        "Finished EROFS extraction for {} into {}",
        image_path.display(),
        destination.display()
    );
    Ok(())
}

fn ensure_fsck_erofs_available() -> Result<()> {
    let version = ensure_fsck_erofs_available_with(FSCK_EROFS_BINARY)?;
    if version.is_empty() {
        info!("Found fsck.erofs in PATH");
    } else {
        info!("Found {}", version);
    }
    Ok(())
}

fn ensure_fsck_erofs_available_with(binary: &str) -> Result<String> {
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .map_err(|error| fsck_erofs_command_error(binary, error))?;

    ensure!(
        output.status.success(),
        "{} --version failed with {}",
        binary,
        format_exit_status(output.status)
    );

    let version = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr).trim().to_owned()
    } else {
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    };

    Ok(version)
}

fn run_fsck_erofs_extract(image_path: &Path, destination: &Path) -> Result<()> {
    let extract_arg = extract_arg(destination);
    debug!(
        "Executing command: {} {} --overwrite --no-preserve-owner --preserve-perms {}",
        FSCK_EROFS_BINARY,
        extract_arg.to_string_lossy(),
        image_path.display()
    );

    let status = Command::new(FSCK_EROFS_BINARY)
        .arg(&extract_arg)
        .arg("--overwrite")
        .arg("--no-preserve-owner")
        .arg("--preserve-perms")
        .arg(image_path)
        .status()
        .map_err(|error| fsck_erofs_command_error(FSCK_EROFS_BINARY, error))?;

    ensure!(
        status.success(),
        "fsck.erofs failed with {} while extracting {}",
        format_exit_status(status),
        image_path.display()
    );
    Ok(())
}

fn extract_arg(destination: &Path) -> OsString {
    let mut arg = OsString::from("--extract=");
    arg.push(destination);
    arg
}

fn fsck_erofs_command_error(binary: &str, error: io::Error) -> anyhow::Error {
    match error.kind() {
        io::ErrorKind::NotFound => anyhow!(
            "{binary} was not found in PATH; install erofs-utils before extracting EROFS images"
        ),
        _ => anyhow!("failed to execute {binary}: {error}"),
    }
}

fn format_exit_status(status: ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("exit status {code}"),
        None => "termination by signal".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{ensure_fsck_erofs_available_with, extract_arg};

    #[test]
    fn builds_extract_argument() {
        let argument = extract_arg(Path::new("/tmp/output dir"));
        assert_eq!(argument.to_string_lossy(), "--extract=/tmp/output dir");
    }

    #[test]
    fn missing_fsck_erofs_binary_has_clear_error() {
        let error =
            ensure_fsck_erofs_available_with("fsck.erofs-definitely-not-installed").unwrap_err();
        assert!(error.to_string().contains("install erofs-utils"));
    }
}
