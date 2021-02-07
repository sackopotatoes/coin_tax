use std::process;
use clap::Clap;

#[derive(Clap)]
struct Opts {
    file: String,
    #[clap(short, long, default_value = "coinbase")]
    exchange: String,
}

fn main() {
    let opts: Opts = Opts::parse();

    if let Err(e) = coin_tax::run(&opts.file, &opts.exchange) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
