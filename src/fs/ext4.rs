use std::{
    ffi::OsString,
    fs, io,
    path::Path,
    process::{Command, ExitStatus},
};

use anyhow::{Context, Result, anyhow, ensure};
use log::{debug, info};

const SEVEN_ZIP_BINARY: &str = "7z";

pub fn extract_image(image_path: &Path, destination: &Path) -> Result<()> {
    info!(
        "Extracting ext4 image {} into {} with 7z",
        image_path.display(),
        destination.display()
    );

    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;
    ensure_seven_zip_available()?;
    run_seven_zip_extract(image_path, destination)?;

    info!(
        "Finished ext4 extraction for {} into {}",
        image_path.display(),
        destination.display()
    );
    Ok(())
}

fn ensure_seven_zip_available() -> Result<()> {
    let version = ensure_seven_zip_available_with(SEVEN_ZIP_BINARY)?;
    if version.is_empty() {
        info!("Found 7z in PATH");
    } else {
        info!("Found {}", version.lines().next().unwrap_or("7z"));
    }
    Ok(())
}

fn ensure_seven_zip_available_with(binary: &str) -> Result<String> {
    let output = Command::new(binary)
        .arg("-h")
        .output()
        .map_err(|error| seven_zip_command_error(binary, error))?;

    ensure!(
        output.status.success(),
        "{} -h failed with {}",
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

fn run_seven_zip_extract(image_path: &Path, destination: &Path) -> Result<()> {
    let output_arg = output_arg(destination);
    debug!(
        "Executing command: {} x -y {} {}",
        SEVEN_ZIP_BINARY,
        output_arg.to_string_lossy(),
        image_path.display()
    );

    let status = Command::new(SEVEN_ZIP_BINARY)
        .arg("x")
        .arg("-y")
        .arg(&output_arg)
        .arg(image_path)
        .status()
        .map_err(|error| seven_zip_command_error(SEVEN_ZIP_BINARY, error))?;

    ensure!(
        status.success(),
        "7z failed with {} while extracting {}",
        format_exit_status(status),
        image_path.display()
    );

    Ok(())
}

fn output_arg(destination: &Path) -> OsString {
    let mut arg = OsString::from("-o");
    arg.push(destination);
    arg
}

fn seven_zip_command_error(binary: &str, error: io::Error) -> anyhow::Error {
    match error.kind() {
        io::ErrorKind::NotFound => {
            anyhow!("{binary} was not found in PATH; install 7zip before extracting ext4 images")
        }
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
    use std::{fs, path::Path, process::Command};

    use anyhow::Result;
    use tempfile::tempdir;

    use super::{ensure_seven_zip_available_with, extract_image, output_arg};

    #[test]
    fn builds_7z_output_argument() {
        let argument = output_arg(Path::new("/tmp/output dir"));
        assert_eq!(argument.to_string_lossy(), "-o/tmp/output dir");
    }

    #[test]
    fn missing_7z_binary_has_clear_error() {
        let error = ensure_seven_zip_available_with("7z-definitely-not-installed").unwrap_err();
        assert!(error.to_string().contains("install 7zip"));
    }

    #[test]
    fn extracts_small_ext4_image_with_7z_when_available() -> Result<()> {
        if !command_available("7z") || !command_available("mkfs.ext4") {
            return Ok(());
        }

        let tempdir = tempdir()?;
        let image_path = tempdir.path().join("sample.img");
        let source_path = tempdir.path().join("hello.txt");
        let destination = tempdir.path().join("output dir");

        std::fs::write(&source_path, "hello from 7z\n")?;
        run_command("truncate", ["-s", "8M", image_path.to_str().unwrap()])?;
        run_command("mkfs.ext4", ["-F", "-q", image_path.to_str().unwrap()])?;
        run_command(
            "debugfs",
            ["-w", "-R", "mkdir /etc", image_path.to_str().unwrap()],
        )?;
        run_command(
            "debugfs",
            [
                "-w",
                "-R",
                &format!("write {} /etc/hello.txt", source_path.display()),
                image_path.to_str().unwrap(),
            ],
        )?;

        extract_image(&image_path, &destination)?;

        assert_eq!(
            fs::read_to_string(destination.join("etc/hello.txt"))?,
            "hello from 7z\n"
        );

        Ok(())
    }

    fn command_available(binary: &str) -> bool {
        Command::new(binary).arg("-h").output().is_ok()
    }

    fn run_command<const N: usize>(binary: &str, args: [&str; N]) -> Result<()> {
        let status = Command::new(binary).args(args).status()?;
        anyhow::ensure!(status.success(), "{binary} failed with {status}");
        Ok(())
    }
}
