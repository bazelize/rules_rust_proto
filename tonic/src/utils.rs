use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use anyhow::{Context, Result};
use bytes::Bytes;
use prost::Message;
use prost_types::FileDescriptorSet;

/// Parses a protobuf descriptor set at the given path.
pub(crate) fn parse_descriptor_set(path: &Path) -> Result<FileDescriptorSet> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    // Read file into vector.
    reader.read_to_end(&mut buffer)?;
    let bytes = Bytes::from(buffer);

    let descriptor_set =
        FileDescriptorSet::decode(bytes).context("Failed to decode filedescriptorset")?;

    Ok(descriptor_set)
}
