use std::process;
use structopt::StructOpt;
use tonic_proto_transpiler::cli;

fn main() {
    let opt = cli::Opt::from_args();

    if let Err(err) = cli::transpile(&opt) {
        eprintln!("Failed while transpiling: {}", err);
        process::exit(1);
    }
}
