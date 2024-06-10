mod error;
mod options;
mod runner;

fn main() {
    let options = <options::Options as clap::Parser>::parse();
    std::process::exit(runner::run(options).unwrap_or_else(Into::into));
}
