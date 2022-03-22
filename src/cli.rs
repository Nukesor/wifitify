use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "Sniff", about = "Track wifi devices", author, version)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// The device you want to listen on (e.g. [wlan0, wlp3s0])
    pub device: String,
}
