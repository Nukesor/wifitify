use std::path::PathBuf;

use clap::Clap;

#[derive(Clap, Debug)]
#[clap(name = "Sniff", about = "Track wifi devices", author, version)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
}
