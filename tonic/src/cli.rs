use anyhow::{bail, Context, Result};
use bytes::Bytes;
use prost::Message;
use prost_types::FileDescriptorSet;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;
use std::process::Command;
use std::{io::Write, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "tonic-transpiler", rename_all = "snake_case")]
pub struct Opt {
    #[structopt(long = "name")]
    pub name: String,
    #[structopt(long = "edition")]
    pub edition: String,
    #[structopt(long = "proto", multiple = true)]
    pub protos: Vec<PathBuf>,
    #[structopt(long = "include", multiple = true)]
    pub includes: Vec<PathBuf>,
    #[structopt(long = "direct_descriptor_set")]
    pub direct_descriptor_set: PathBuf,
    #[structopt(long = "transitive_descriptor_set")]
    pub transitive_descriptor_sets: Vec<PathBuf>,
    #[structopt(long)]
    pub output_directory: PathBuf,

    // TODO: Delete these?
    #[structopt(long)]
    pub client: bool,
    #[structopt(long)]
    pub server: bool,
}

pub fn transpile(opt: &Opt) -> Result<()> {
    run_tonic(opt)?;

    generate_librs(opt)?;
    generate_cargo_workspace(opt)?;

    Ok(())
}

fn run_tonic(opt: &Opt) -> Result<()> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_well_known_types(true)
        .format(true)
        // .client_attribute(path, attribute)
        // .client_mod_attribute(path, attribute)
        // .server_attribute(path, attribute)
        // .server_mod_attribute(path, attribute)
        .out_dir(&opt.output_directory)
        .compile(&opt.protos, &opt.includes)
        .context("Failed to compile protos")?;

    Ok(())
}

fn parse_descriptor_set(path: &Path) -> Result<FileDescriptorSet> {
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

struct RustModule {
    modules: HashMap<String, RustModule>,
    contents: String,
}

impl RustModule {
    fn new() -> Self {
        Self {
            modules: HashMap::new(),
            contents: "".to_string(),
        }
    }

    fn insert_module(&mut self, module_name: &str, contents: String) {
        let mut current_module = self;
        for module_name in module_name.split(".") {
            let module_name = module_name.to_owned();
            if !current_module.modules.contains_key(&module_name) {
                current_module
                    .modules
                    .insert(module_name.clone(), RustModule::new());
            }
            current_module = current_module.modules.get_mut(&module_name).unwrap();
        }

        current_module.contents = contents;
    }

    fn serialize(&self, depth: usize) -> Result<String> {
        let mut contents = Vec::new();

        for (module_name, module) in self.modules.iter() {
            contents.push(format!(
                "pub mod {} {{\n{}\n}}",
                module_name,
                &module.serialize(depth + 1)?
            ))
        }

        contents.push(self.contents.clone());

        Ok(contents.join("\n"))
    }
}

fn generate_librs(opt: &Opt) -> Result<()> {
    let librs = opt.output_directory.join("lib.rs");

    let direct_descriptor_set = parse_descriptor_set(&opt.direct_descriptor_set)?;
    let transitive_descriptor_sets: Vec<FileDescriptorSet> = opt
        .transitive_descriptor_sets
        .iter()
        .map(|p| parse_descriptor_set(p).unwrap())
        .collect();

    let mut librs_module = RustModule::new();

    let mut package_set = HashSet::new();

    for descriptor_set in transitive_descriptor_sets.iter() {
        for file_descriptor in descriptor_set.file.iter() {
            let package_name = file_descriptor
                .package
                .as_ref()
                .context("No package specified")?;
            if package_set.contains(package_name) {
                continue;
            }
            package_set.insert(
                file_descriptor
                    .package
                    .as_ref()
                    .context("No package specified")?,
            );

            let rust_module_contents =
                &fs::read_to_string(opt.output_directory.join(format!("{}.rs", package_name)))
                    .context("Failed to read generated rust module")?;
            librs_module.insert_module(package_name, rust_module_contents.to_owned());
        }
    }

    for file_descriptor in direct_descriptor_set.file.iter() {
        let package_name = file_descriptor
            .package
            .as_ref()
            .context("No package specified")?;
        package_set.insert(package_name);

        let rust_module_contents =
            &fs::read_to_string(opt.output_directory.join(format!("{}.rs", package_name)))
                .context("Failed to read generated rust module")?;
        librs_module.insert_module(package_name, rust_module_contents.to_owned());
    }

    fs::File::create(&librs)
        .context("could not create lib.rs file")?
        .write_all(librs_module.serialize(0)?.as_bytes())
        .context("could not write to lib.rs file")?;

    let rustfmt_bin = std::env::var("RUSTFMT").context("Could not find RUSTFMT variable.")?;
    let output = Command::new(rustfmt_bin)
        .arg(format!("--edition={}", &opt.edition))
        .arg(&librs)
        .output()
        .expect("Failed to execute command");
    if !output.status.success() {
        bail!(
            "Failed to run RUSTFMT:\nstdout:\n{}\nstderr:\n{}",
            std::str::from_utf8(&output.stdout).unwrap(),
            std::str::from_utf8(&output.stderr).unwrap()
        );
    }

    Ok(())
}

fn generate_cargo_workspace(opt: &Opt) -> Result<()> {
    let cargo_toml = opt.output_directory.join("Cargo.toml");
    let contents = format!(r#"
[package]
name = "{}"
version = "1.0.0"
edition = "{}"

[lib]
path = "lib.rs"

[dependencies]
tonic = "0"
prost = "0"
    "#, &opt.name, &opt.edition);

    fs::File::create(cargo_toml)
        .context("could not create cargo.toml file")?
        .write_all(contents.as_bytes())
        .context("could not write to cargo.toml file")?;

    Ok(())
}
