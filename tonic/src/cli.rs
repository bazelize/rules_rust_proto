use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::generator;

/// Tonic transpiler command line options.
#[derive(Debug, StructOpt)]
#[structopt(name = "Tonic Transpiler", rename_all = "snake_case")]
pub struct Opt {
    #[structopt(long, help = "The name of the transpiled library.")]
    pub name: String,
    #[structopt(long, help = "The rust edition.")]
    pub edition: String,
    #[structopt(long = "direct_source", multiple = true, help = "The proto's direct source files.")]
    pub direct_sources: Vec<PathBuf>,
    #[structopt(long = "proto_path", multiple = true, help = "The proto's transitive proto import path.")]
    pub transitive_proto_path: Vec<PathBuf>,
    #[structopt(long = "direct_descriptor_set", help = "The proto's direct file descriptor set path.")]
    pub direct_descriptor_set: PathBuf,
    #[structopt(long = "transitive_descriptor_set", help = "The proto's transitive descriptor set paths.")]
    pub transitive_descriptor_sets: Vec<PathBuf>,
    #[structopt(long, help = "The output lib.rs to be generated.")]
    pub output_librs: PathBuf,
    #[structopt(long, help = "The output Cargo.toml to be generated.")]
    pub output_cargo_toml: PathBuf,
}

/// Generates the lib.rs and Cargo.toml files using Tonic.
pub fn transpile(opt: &Opt) -> Result<()> {
    generator::generate_librs(opt)?;
    generator::generate_cargo_workspace(opt)?;

    Ok(())
}
