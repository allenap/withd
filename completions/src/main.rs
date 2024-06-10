use std::{env, fs, io, path::PathBuf};

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

mod options {
    include!("../../withd/src/options.rs");
}

fn main() -> Result<(), io::Error> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let scripts_dir = PathBuf::from(&manifest_dir).join("scripts");
    fs::remove_dir_all(&scripts_dir)?;
    fs::create_dir_all(&scripts_dir)?;

    let mut cmd = options::Options::command();

    let current_dir = env::current_dir()?;
    for shell in <Shell as clap::ValueEnum>::value_variants() {
        let path = generate_to(*shell, &mut cmd, "withd", &scripts_dir)?;
        let path_relative = path.strip_prefix(&current_dir).unwrap();
        println!("Completion file for {shell} generated @ {path_relative:?}");
    }

    Ok(())
}
