use std::{env, fs, io, path::PathBuf};

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

mod options {
    include!("src/options.rs");
}

fn main() -> Result<(), io::Error> {
    let manifest_dir = env::var_os("OUT_DIR").expect("OUT_DIR is not set");
    let completions_dir = PathBuf::from(manifest_dir).join("completions");
    fs::create_dir_all(&completions_dir)?;

    let mut cmd = options::Options::command();
    let bin_name = cmd.get_name().to_owned();

    for shell in <Shell as clap::ValueEnum>::value_variants() {
        let path = generate_to(*shell, &mut cmd, &bin_name, &completions_dir)?;
        println!("cargo:warning=completion file ({shell}) generated: {path:?}");
    }

    Ok(())
}
