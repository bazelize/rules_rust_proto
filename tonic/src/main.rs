use structopt::StructOpt;
use tonic_proto_transpiler::cli;
fn main() {
    let opt = cli::Opt::from_args();

    cli::transpile(&opt).unwrap();
}
