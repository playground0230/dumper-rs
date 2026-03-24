use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use log::{debug, info};
use sha1::{Digest, Sha1};

const ALL_FILES_MANIFEST: &str = "all_files.txt";
const ALL_FILES_SHA1_MANIFEST: &str = "all_files.sha1sum.txt";
const SHA1_PROGRESS_INTERVAL: usize = 1000;

pub fn write_manifests(root: &Path, include_sha1: bool) -> Result<()> {
    info!("Scanning extracted files under {}", root.display());
    let mut files = Vec::new();
    collect_regular_files(root, root, &mut files)?;
    files.sort();
    info!(
        "Collected {} regular file(s) for manifest generation",
        files.len()
    );

    let all_files_path = root.join(ALL_FILES_MANIFEST);
    info!("Writing {}", all_files_path.display());
    let mut all_files = File::create(&all_files_path)
        .with_context(|| format!("failed to create {}", all_files_path.display()))?;

    for path in &files {
        debug!("Adding manifest entry {}", path);
        writeln!(all_files, "{path}")
            .with_context(|| format!("failed to write {}", all_files_path.display()))?;
    }

    if include_sha1 {
        let sha1_path = root.join(ALL_FILES_SHA1_MANIFEST);
        info!("Writing {} with SHA-1 digests", sha1_path.display());
        let mut sha1_file = File::create(&sha1_path)
            .with_context(|| format!("failed to create {}", sha1_path.display()))?;

        for (index, relative_path) in files.iter().enumerate() {
            if index == 0 || (index + 1) % SHA1_PROGRESS_INTERVAL == 0 || index + 1 == files.len() {
                info!(
                    "SHA-1 progress: {}/{} file(s) processed",
                    index + 1,
                    files.len()
                );
            }
            debug!("Hashing {}", relative_path);
            let digest = sha1_file_digest(&root.join(relative_path))?;
            writeln!(sha1_file, "{relative_path}|{digest}")
                .with_context(|| format!("failed to write {}", sha1_path.display()))?;
        }
    }

    info!("Manifest generation complete");
    Ok(())
}

fn collect_regular_files(root: &Path, current: &Path, out: &mut Vec<String>) -> Result<()> {
    let mut entries = fs::read_dir(current)
        .with_context(|| format!("failed to read {}", current.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("failed to enumerate {}", current.display()))?;

    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?;
        let path = entry.path();

        if file_type.is_dir() {
            collect_regular_files(root, &path, out)?;
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let relative_path = path
            .strip_prefix(root)
            .with_context(|| format!("failed to relativize {}", path.display()))?;
        let normalized = normalize_relative_path(relative_path);

        if normalized == ALL_FILES_MANIFEST || normalized == ALL_FILES_SHA1_MANIFEST {
            continue;
        }

        out.push(normalized);
    }

    Ok(())
}

fn normalize_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn sha1_file_digest(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;

    use tempfile::tempdir;

    use super::write_manifests;

    #[test]
    fn writes_sorted_manifests_and_skips_manifest_files() {
        let tempdir = tempdir().unwrap();
        let root = tempdir.path();

        fs::create_dir_all(root.join("vendor/bin")).unwrap();
        fs::create_dir_all(root.join("system/bin")).unwrap();

        File::create(root.join("vendor/bin/sh")).unwrap();

        let mut second = File::create(root.join("system/bin/app_process")).unwrap();
        writeln!(second, "hello world").unwrap();

        File::create(root.join("all_files.txt")).unwrap();
        File::create(root.join("all_files.sha1sum.txt")).unwrap();

        write_manifests(root, true).unwrap();

        let all_files = fs::read_to_string(root.join("all_files.txt")).unwrap();
        assert_eq!(all_files, "system/bin/app_process\nvendor/bin/sh\n");

        let sha1sum = fs::read_to_string(root.join("all_files.sha1sum.txt")).unwrap();
        assert!(sha1sum.starts_with("system/bin/app_process|"));
        assert!(sha1sum.contains("\nvendor/bin/sh|"));
    }
}
