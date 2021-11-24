use anyhow::{bail, Context, Result};
use prost_types::FileDescriptorSet;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::cli::Opt;
use crate::utils::parse_descriptor_set;

/// Represents a generated Rust module. Used for managing and serializing rust modules.
struct RustModule {
    modules: HashMap<String, RustModule>,
    contents: String,
}

impl RustModule {
    /// Create a new RustModule with default values.
    fn new() -> Self {
        Self {
            modules: HashMap::new(),
            contents: "".to_string(),
        }
    }

    /// Insert a module with the given contents at the given module path.
    fn insert_module(&mut self, module_name: &str, contents: String) -> Result<()> {
        let mut current_module = self;
        for module_name in module_name.split(".") {
            let module_name = module_name.to_owned();
            if !current_module.modules.contains_key(&module_name) {
                current_module
                    .modules
                    .insert(module_name.clone(), RustModule::new());
            }
            current_module = current_module.modules.get_mut(&module_name).context("Failed to get module by name")?;
        }

        current_module.contents = contents;

        Ok(())
    }

    /// Serialize the RustModule to a string.
    fn serialize(&self) -> Result<String> {
        let mut contents = Vec::new();

        // Write all of the modules in alphabetical order.
        let mut module_names: Vec<&String> = self.modules.keys().collect();
        module_names.sort();

        for module_name in module_names {
            let module = self.modules.get(module_name).context("Failed to find module")?;
            contents.push(format!(
                "pub mod {} {{\n{}\n}}",
                module_name,
                &module.serialize()?
            ))
        }

        contents.push(self.contents.clone());

        Ok(contents.join("\n"))
    }
}

/// Generate the Rust lib.rs file.
pub fn generate_librs(opt: &Opt) -> Result<()> {
    run_tonic(opt)?;

    let direct_descriptor_set = parse_descriptor_set(&opt.direct_descriptor_set)?;
    let transitive_descriptor_sets = opt
        .transitive_descriptor_sets
        .iter()
        .map(|p| parse_descriptor_set(p))
        .collect::<Result<Vec<FileDescriptorSet>>>()?;

    let mut librs_module = RustModule::new();
    let mut package_set = HashSet::new();

    for descriptor_set in transitive_descriptor_sets
        .iter()
        .chain([&direct_descriptor_set])
    {
        for file_descriptor in descriptor_set.file.iter() {
            let package_name = file_descriptor
                .package
                .as_ref()
                .context("No package specified")?;
            if package_set.contains(package_name) {
                continue;
            }

            package_set.insert(package_name);

            let package_file_name = opt
                .output_librs
                .with_file_name(format!("{}.rs", package_name));
            let rust_module_contents = fs::read_to_string(package_file_name)
                .context("Failed to read generated rust module")?;
            librs_module.insert_module(package_name, rust_module_contents)?;
        }
    }

    fs::File::create(&opt.output_librs)
        .context("could not create lib.rs file")?
        .write_all(librs_module.serialize()?.as_bytes())
        .context("could not write to lib.rs file")?;

    run_rustfmt(&opt.output_librs, &opt.edition)?;

    Ok(())
}

/// Run the tonic generator.
fn run_tonic(opt: &Opt) -> Result<()> {
    let output_directory = opt
        .output_librs
        .parent()
        .context("Cannot get parent of input lib.rs")?;

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_well_known_types(true)
        .format(true)
        .out_dir(&output_directory)
        .compile(&opt.direct_sources, &opt.transitive_proto_path)
        .context("Failed to compile protos")?;

    Ok(())
}

/// Run rustfmt, with the given editioin, on the given rust file.
fn run_rustfmt(rust_file: &Path, edition: &str) -> Result<()> {
    let rustfmt_bin = std::env::var("RUSTFMT").context("Could not find RUSTFMT variable.")?;
    let output = Command::new(rustfmt_bin)
        .arg(format!("--edition={}", &edition))
        .arg(&rust_file)
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

/// Generates the Cargo.toml file.
pub fn generate_cargo_workspace(opt: &Opt) -> Result<()> {
    let contents = format!(
        r#"
[package]
name = "{}"
version = "1.0.0"
edition = "{}"

[lib]
path = "lib.rs"

[dependencies]
tonic = "0"
prost = "0"
    "#,
        &opt.name, &opt.edition
    );

    fs::File::create(&opt.output_cargo_toml)
        .context("could not create cargo.toml file")?
        .write_all(contents.as_bytes())
        .context("could not write to cargo.toml file")?;

    Ok(())
}
