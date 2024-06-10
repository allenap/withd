use std::{env, fs, io, path::PathBuf};

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

mod options;

fn main() -> Result<(), io::Error> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let completions_dir = PathBuf::from(&manifest_dir).join("completions");
    fs::remove_dir_all(&completions_dir)?;
    fs::create_dir_all(&completions_dir)?;

    let mut cmd = options::Options::command();

    let current_dir = env::current_dir()?;
    for shell in <Shell as clap::ValueEnum>::value_variants() {
        let path = generate_to(*shell, &mut cmd, "withd", &completions_dir)?;
        let path_relative = path.strip_prefix(&current_dir).unwrap();
        println!("Completion file for {shell} generated @ {path_relative:?}");
    }

    Ok(())
}
