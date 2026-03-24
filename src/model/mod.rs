use std::path::PathBuf;

use crate::cli::CliArgs;

const FILESYSTEM_DUMP_PARTITIONS: &[&str] = &[
    "system",
    "systemex",
    "system_ext",
    "system_other",
    "vendor",
    "cust",
    "odm",
    "odm_ext",
    "oem",
    "factory",
    "product",
    "modem",
    "xrom",
    "oppo_product",
    "opproduct",
    "reserve",
    "india",
    "my_preload",
    "my_odm",
    "my_stock",
    "my_operator",
    "my_country",
    "my_product",
    "my_company",
    "my_engineering",
    "my_heytap",
    "my_custom",
    "my_manifest",
    "my_carrier",
    "my_region",
    "my_bigball",
    "my_version",
    "special_preload",
    "vendor_dlkm",
    "odm_dlkm",
    "system_dlkm",
    "mi_ext",
    "radio",
    "product_h",
    "preas",
    "preavs",
    "preload",
];

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub input_zip: PathBuf,
    pub output_dir_override: Option<PathBuf>,
    pub all_files_sha1: bool,
}

impl From<CliArgs> for AppConfig {
    fn from(args: CliArgs) -> Self {
        Self {
            input_zip: args.input_zip,
            output_dir_override: args.output_dir,
            all_files_sha1: args.all_files_sha1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutputLayout {
    pub root: PathBuf,
    pub images_dir: PathBuf,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PartitionImage {
    pub name: String,
    pub image_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FilesystemKind {
    Ext4,
    Erofs,
}

pub fn should_dump_partition_files(partition_name: &str) -> bool {
    FILESYSTEM_DUMP_PARTITIONS.contains(&partition_name)
}

#[cfg(test)]
mod tests {
    use super::should_dump_partition_files;

    #[test]
    fn partition_dump_allowlist_matches_expected_behavior() {
        assert!(should_dump_partition_files("system"));
        assert!(should_dump_partition_files("vendor_dlkm"));
        assert!(should_dump_partition_files("preload"));
        assert!(!should_dump_partition_files("boot"));
        assert!(!should_dump_partition_files("vbmeta"));
        assert!(!should_dump_partition_files("dtbo"));
    }
}
