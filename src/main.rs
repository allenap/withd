use std::env::set_current_dir;
use std::fs::create_dir_all;
use std::io::{Error, ErrorKind, Result};
use std::process::{exit, Command};
use std::{ffi::OsString, path::PathBuf};

use clap::{command, Parser};

#[derive(Parser)]
#[command(
    author, version, about, long_about = None,
    after_help = "Execute a command in a specific directory.",
)]
struct Options {
    #[arg(help = "The directory in which to execute the command.")]
    directory: PathBuf,

    #[arg(
        short,
        long,
        help = "Create the directory if it does not exist.",
        default_value_t = false
    )]
    create: bool,

    #[arg(help = "The command to execute.")]
    command: OsString,

    #[arg(
        help = "The arguments to pass to the command.",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    args: Vec<OsString>,
}

fn main() {
    let options = Options::parse();
    exit(run(options).unwrap_or_else(io_error_to_exit_code));
}

#[cfg(unix)]
/// Execute the command in the specified directory. This version is specific to
/// UNIX; specifically, it `exec`s the command, which means that the current
/// process is replaced.
fn run(options: Options) -> Result<i32> {
    use std::os::unix::process::CommandExt;
    // Change to the requested directory.
    change_directory(&options.directory, options.create)?;
    // If everything works correctly, `exec` will diverge.
    let error = Command::new(&options.command).args(&options.args).exec();
    // When `exec` fails, we print its error message.
    eprintln!(
        "Could not execute `{}`: {error}",
        options.command.to_string_lossy()
    );
    Err(error)
}

#[cfg(not(unix))]
/// Execute the command in the specified directory. Works on any platform.
fn run(options: Options) -> Result<i32> {
    // Change to the requested directory before executing the command.
    change_directory(&options.directory, options.create)?;
    match Command::new(&options.command).args(&options.args).spawn() {
        Ok(mut child) => match child.wait() {
            Ok(status) => Ok(status.code().unwrap_or(
                // In Bash (https://www.gnu.org/software/bash/), this would be
                // 128 + signal number. We're not on UNIX and we do not know a
                // signal number – indeed, that concept may not hold here – so
                // we go with 130, which corresponds to SIGINT on UNIX 🤷
                130,
            )),
            Err(error) => {
                // Not entirely sure how we might get here.
                eprintln!(
                    "Could not wait for `{}`: {error}",
                    options.command.to_string_lossy()
                );
                Err(error)
            }
        },
        Err(error) => {
            // Presumably the executable wasn't found, or we don't have
            // permission to execute the named command.
            eprintln!(
                "Could not execute `{}`: {error}",
                options.command.to_string_lossy()
            );
            Err(error)
        }
    }
}

/// Change the current working directory to the specified directory, creating
/// that directory if requested.
fn change_directory(directory: &PathBuf, create: bool) -> Result<()> {
    if create {
        create_dir_all(directory)?
    }
    match set_current_dir(directory) {
        Ok(()) => Ok(()),
        Err(error) => {
            eprintln!(
                "Could not change directory to `{}`: {error}",
                directory.to_string_lossy()
            );
            Err(error)
        }
    }
}

/// Return an exit code based on observed behaviour of Bash
/// (https://www.gnu.org/software/bash/).
fn io_error_to_exit_code(error: Error) -> i32 {
    match error.kind() {
        ErrorKind::PermissionDenied => 126,
        ErrorKind::NotFound => 127,
        _ => 1,
    }
}
