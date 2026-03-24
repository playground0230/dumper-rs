use std::{fs::File, io::Read, path::Path};

use anyhow::{Context, Result};
use log::debug;

use crate::model::FilesystemKind;

const EROFS_MAGIC_OFFSET: usize = 1024;
const EROFS_MAGIC: u32 = 0xe0f5_e1e2;
const EXT4_MAGIC_OFFSET: usize = 1024 + 56;
const EXT4_MAGIC: u16 = 0xef53;

pub fn detect_filesystem_kind(image_path: &Path) -> Result<Option<FilesystemKind>> {
    let mut file = File::open(image_path)
        .with_context(|| format!("failed to open {}", image_path.display()))?;
    let mut header = [0_u8; 2048];
    let read = file
        .read(&mut header)
        .with_context(|| format!("failed to read {}", image_path.display()))?;
    let header = &header[..read];

    if header.len() >= EROFS_MAGIC_OFFSET + 4 {
        let magic = u32::from_le_bytes(
            header[EROFS_MAGIC_OFFSET..EROFS_MAGIC_OFFSET + 4]
                .try_into()
                .expect("slice length checked"),
        );
        if magic == EROFS_MAGIC {
            debug!("Detected EROFS filesystem in {}", image_path.display());
            return Ok(Some(FilesystemKind::Erofs));
        }
    }

    if header.len() >= EXT4_MAGIC_OFFSET + 2 {
        let magic = u16::from_le_bytes(
            header[EXT4_MAGIC_OFFSET..EXT4_MAGIC_OFFSET + 2]
                .try_into()
                .expect("slice length checked"),
        );
        if magic == EXT4_MAGIC {
            debug!("Detected ext4 filesystem in {}", image_path.display());
            return Ok(Some(FilesystemKind::Ext4));
        }
    }

    debug!(
        "Could not detect a supported filesystem type for {}",
        image_path.display()
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use tempfile::tempdir;

    use super::detect_filesystem_kind;
    use crate::model::FilesystemKind;

    #[test]
    fn detects_erofs() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("system.img");
        let mut data = vec![0_u8; 2048];
        data[1024..1028].copy_from_slice(&0xe0f5_e1e2_u32.to_le_bytes());
        fs::File::create(&path).unwrap().write_all(&data).unwrap();

        let kind = detect_filesystem_kind(&path).unwrap();
        assert_eq!(kind, Some(FilesystemKind::Erofs));
    }

    #[test]
    fn detects_ext4() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("vendor.img");
        let mut data = vec![0_u8; 2048];
        data[1080..1082].copy_from_slice(&0xef53_u16.to_le_bytes());
        fs::File::create(&path).unwrap().write_all(&data).unwrap();

        let kind = detect_filesystem_kind(&path).unwrap();
        assert_eq!(kind, Some(FilesystemKind::Ext4));
    }
}
