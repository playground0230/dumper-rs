# dumper-rs

`dumper-rs` is a Rust tool for extracting files from Android OTA packages and generating file manifests.

## Description

- Uses a modified [`otaripper`](https://github.com/playground0230/otaripper) to extract partition images from supported OTA packages.
- Extracts files from supported filesystem images.
- Generates deterministic file manifests from the extracted output.

## Supported Filesystems

- ext4
- EROFS

## Limitations

- Only OTA zip files that contain `payload.bin` are supported.
- Filesystem extraction is currently limited to ext4 and EROFS.

## TODO

- Add support for fastboot zips.
- Add support for vendor-specific formats.
