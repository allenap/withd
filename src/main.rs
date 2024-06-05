use std::env::set_current_dir;
use std::fs::create_dir_all;
use std::io::{Error, ErrorKind, Result};
use std::process::{exit, Command};
use std::{ffi::OsString, path::PathBuf};

use bstr::ByteSlice;
use clap::{command, Parser};
use tempfile::TempDir;

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

    #[arg(
        short,
        long,
        help = "Create a temporary directory within the specified directory.",
        default_value_t = false
    )]
    temporary: bool,

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

/// Execute the command in the specified directory. Works on any platform.
fn run(options: Options) -> Result<i32> {
    // Change to the requested directory before executing the command.
    let _guard = change_directory(&options.directory, options.create, options.temporary)?;
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
                eprintln!("Could not wait for {:?}: {error}", options.command);
                Err(error)
            }
        },
        Err(error) => {
            // Presumably the executable wasn't found, or we don't have
            // permission to execute the named command.
            eprintln!("Could not execute `{:?}`: {error}", options.command);
            Err(error)
        }
    }
}

/// Change the current working directory to the specified directory, creating
/// that directory if requested.
fn change_directory(directory: &PathBuf, create: bool, temporary: bool) -> Result<Option<TempDir>> {
    if temporary {
        let (directory, prefix) = match directory.file_name() {
            None => (Some(directory.as_path()), None),
            Some(name) => match <[u8]>::from_os_str(name) {
                None => {
                    eprintln!("Directory name contains un-decodable parts");
                    return Err(Error::from(ErrorKind::InvalidInput));
                }
                Some(name) => match name.strip_suffix(b"XXXXXX") {
                    None => (Some(directory.as_path()), None),
                    Some(prefix) => match prefix.to_os_str() {
                        Ok(prefix) => (directory.parent(), Some(prefix)),
                        Err(error) => {
                            eprintln!("Directory name contains un-encodable parts");
                            return Err(Error::new(ErrorKind::InvalidInput, error));
                        }
                    },
                },
            },
        };
        if create {
            if let Some(directory) = directory {
                create_dir_all(directory).inspect_err(|error| {
                    eprintln!("Could not create directory {directory:?}: {error}")
                })?
            }
        }
        let tempdir = match (directory, prefix) {
            (Some(directory), Some(prefix)) => TempDir::with_prefix_in(prefix, directory)
                .inspect_err(|error| {
                    eprintln!("Could not create temporary directory with prefix {prefix:?} in {directory:?}: {error}")
                }),
            (Some(directory), None) => TempDir::new_in(directory)
                .inspect_err(|error| eprintln!("Could not create temporary directory in {directory:?}: {error}")),
            (None, Some(prefix)) => TempDir::with_prefix(prefix)
                .inspect_err(|error| eprintln!("Could not create temporary directory with prefix {prefix:?}: {error}")),
            (None, None) => TempDir::new()
                .inspect_err(|error| eprintln!("Could not create temporary directory: {error}")),
        }?;
        set_current_dir(&tempdir).inspect_err(|error| {
            eprintln!("Could not change directory to `{tempdir:?}`: {error}")
        })?;
        Ok(Some(tempdir))
    } else {
        if create {
            create_dir_all(directory).inspect_err(|error| {
                eprintln!("Could not create directory {directory:?}: {error}")
            })?
        }
        set_current_dir(directory).inspect_err(|error| {
            eprintln!("Could not change directory to `{directory:?}`: {error}")
        })?;
        Ok(None)
    }
}

/// Return an exit code based on observed behaviour of Bash
/// (https://www.gnu.org/software/bash/).
fn io_error_to_exit_code(error: Error) -> i32 {
    match error.kind() {
        // [unstable] ErrorKind::ReadOnlyFilesystem => 30,
        ErrorKind::PermissionDenied => 126,
        ErrorKind::NotFound => 127,
        _ => 1,
    }
}
