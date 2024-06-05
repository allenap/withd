use std::env::{self, set_current_dir};
use std::fs::create_dir_all;
use std::io;
use std::process::{exit, Command};
use std::{ffi::OsString, path::Path, path::PathBuf};

use bstr::ByteSlice;
use clap::{command, Parser};
use lazy_regex::bytes_regex_captures;
use tempfile::{Builder as TempBuilder, TempDir};

#[derive(Parser)]
#[command(
    author, version, about, long_about = None,
    after_help = "Execute a command in a specific directory.",
)]
struct Options {
    #[arg(help = "The directory in which to execute the command.")]
    // NOTE: This is `OsString` because, at present, `clap` does not allow for
    // empty `PathBuf` values – it immediately raises an error.
    directory: OsString,

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
    exit(run(options).unwrap_or_else(|error| error.into()));
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] bstr::Utf8Error),
}

impl From<Error> for i32 {
    fn from(error: Error) -> i32 {
        match error {
            Error::IoError(error) => io_error_to_exit_code(error),
            Error::Utf8Error(_) => 1,
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Execute the command in the specified directory. Works on any platform.
fn run(options: Options) -> Result<i32> {
    // Change to the requested directory before executing the command.
    let _guard = change_directory(&options.directory.into(), options.create, options.temporary)?;
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
                Err(error)?
            }
        },
        Err(error) => {
            // Presumably the executable wasn't found, or we don't have
            // permission to execute the named command.
            eprintln!("Could not execute {:?}: {error}", options.command);
            Err(error)?
        }
    }
}

/// Change the current working directory to the specified directory, creating
/// that directory if requested.
fn change_directory(path: &PathBuf, create: bool, temporary: bool) -> Result<Option<TempDir>> {
    if temporary {
        // The methods `from_os_str` and `to_os_str` below come from
        // `bstr::ByteSlice`. This DTRT on UNIX and Windows.
        let (directory, builder): (_, TempBuilder) = match path.file_name() {
            None => (Some(path.as_path()), TempBuilder::new()),
            Some(name) => match <[u8]>::from_os_str(name) {
                None => {
                    eprintln!("Directory name contains un-decodable parts");
                    return Err(io::Error::from(io::ErrorKind::InvalidInput))?;
                }
                Some(name) => match bytes_regex_captures!(r"^(.*?)(X+)(.*?)$", name) {
                    None => (Some(path.as_path()), TempBuilder::new()),
                    Some((_, prefix, pattern, suffix)) => {
                        let mut builder = TempBuilder::new();
                        builder
                            .prefix(prefix.to_os_str()?)
                            .rand_bytes(pattern.len())
                            .suffix(suffix.to_os_str()?);
                        (path.parent(), builder)
                    }
                },
            },
        };
        let directory = directory.and_then(squash_empty_path);
        if let (Some(directory), true) = (directory, create) {
            create_dir_all(directory).inspect_err(|error| {
                eprintln!("Could not create directory {directory:?}: {error}")
            })?
        }
        let directory = directory.map(Path::to_owned).unwrap_or_else(env::temp_dir);
        let tempdir = builder.tempdir_in(&directory).inspect_err(|error| {
            eprintln!("Could not create temporary directory in {directory:?}: {error}")
        })?;
        set_current_dir(&tempdir)
            .inspect_err(|error| eprintln!("Could not change directory to {tempdir:?}: {error}"))?;
        Ok(Some(tempdir))
    } else {
        if create {
            create_dir_all(path)
                .inspect_err(|error| eprintln!("Could not create directory {path:?}: {error}"))?
        }
        set_current_dir(path)
            .inspect_err(|error| eprintln!("Could not change directory to {path:?}: {error}"))?;
        Ok(None)
    }
}

/// Return an exit code based on observed behaviour of Bash
/// (https://www.gnu.org/software/bash/).
fn io_error_to_exit_code(error: io::Error) -> i32 {
    match error.kind() {
        // [unstable] io::ErrorKind::ReadOnlyFilesystem => 30,
        io::ErrorKind::PermissionDenied => 126,
        io::ErrorKind::NotFound => 127,
        _ => 1,
    }
}

/// Squash an empty path to `None`.
fn squash_empty_path(path: &Path) -> Option<&Path> {
    if path.iter().next().is_some() {
        Some(path)
    } else {
        None
    }
}
