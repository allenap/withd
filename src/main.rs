use std::env::{self, set_current_dir};
use std::process::{exit, Command};
use std::{ffi::OsString, path::Path};
use std::{fs::create_dir_all, io};

use bstr::ByteSlice;
use clap::Parser;
use lazy_regex::bytes_regex_captures;
use tempfile::{Builder as TempBuilder, TempDir};

#[cfg(unix)]
use nix::sys::signal::{signal, SigHandler, Signal};

mod options;

fn main() {
    let options = options::Options::parse();
    exit(run(options).unwrap_or_else(Into::into));
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] bstr::Utf8Error),
    #[error("UTF-8 encoding invalid: {0:?}")]
    Utf8Invalid(OsString),
    #[error("Command terminated by signal ({0})")]
    TerminatedBySignal(i32),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[cfg(unix)]
    #[error("OS error: {0}")]
    Os(#[from] nix::Error),
}

/// Allow for easy conversion of errors to exit codes.
impl From<Error> for i32 {
    fn from(error: Error) -> i32 {
        match error {
            Error::Utf8(_) => 1,
            Error::Utf8Invalid(_) => 1,
            Error::TerminatedBySignal(signo) => {
                // Bash (https://www.gnu.org/software/bash/) uses the exit code
                // 128 + signal number.
                128 + signo
            }
            Error::Io(error) => {
                // Codes based on observed behaviour of Bash
                // (https://www.gnu.org/software/bash/).
                match error.kind() {
                    // [unstable] io::ErrorKind::ReadOnlyFilesystem => 30,
                    io::ErrorKind::PermissionDenied => 126,
                    io::ErrorKind::NotFound => 127,
                    _ => 1,
                }
            }
            #[cfg(unix)]
            Error::Os(errno) => {
                // Similar to `Error::Io`, but using `nix::errno::Errno`, this
                // mimics what Bash might do.
                match errno {
                    nix::errno::Errno::EPERM => 126,
                    nix::errno::Errno::ENOENT => 127,
                    _ => 1,
                }
            }
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

/// The signals that this program will ignore. They'll be reenabled for child
/// processes so that they can receive them as usual – and be terminated most
/// likely – after which this program can clean up.
#[cfg(unix)]
const TERMINATION_SIGNALS: &[Signal] = &[Signal::SIGINT, Signal::SIGQUIT, Signal::SIGTERM];

/// Execute the command in the specified directory. Works on any platform.
fn run(options: options::Options) -> Result<i32> {
    // Ignore signals so that we're not terminated by them. We'll reenable them
    // later on in the child process, just before exec'ing.
    #[cfg(unix)]
    for sig in TERMINATION_SIGNALS {
        unsafe {
            signal(*sig, SigHandler::SigIgn)?;
        }
    }

    // Change to the requested directory before executing the command. We keep
    // the guard value around because it might be a temporary directory that
    // deletes itself on [`Drop`].
    let _guard = if options.temporary {
        ensure_temporary_directory(&options.directory, options.create).map(Some)?
    } else {
        ensure_directory(&options.directory, options.create).map(|_| None)?
    };

    // Prepare the command.
    let mut command = Command::new(&options.command);
    command.args(&options.args);

    // Reset signal handlers for the child process.
    #[cfg(unix)]
    unsafe {
        use std::os::unix::process::CommandExt;
        command.pre_exec(|| {
            for sig in TERMINATION_SIGNALS {
                signal(*sig, SigHandler::SigDfl)?;
            }
            Ok(())
        });
    }

    // Execute the command.
    let mut child = command.spawn().inspect_err(|error| {
        // Presumably the executable wasn't found, or we don't have permission
        // to execute the named command.
        eprintln!("Could not execute {:?}: {error}", options.command);
    })?;

    match child.wait() {
        Ok(status) => match status.code() {
            Some(code) => Ok(code),
            None => {
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    match status.signal() {
                        Some(signo) => Err(Error::TerminatedBySignal(signo)),
                        None => unreachable!("No exit code or signal"),
                    }
                }
                #[cfg(not(unix))]
                {
                    // We're not on UNIX and we do not know a signal number –
                    // indeed, that concept may not hold here – so we go with 2,
                    // which corresponds to SIGINT on UNIX 🤷
                    Err(Error::TerminatedBySignal(2))
                }
            }
        },
        Err(error) => {
            // Not entirely sure how we might get here.
            eprintln!("Could not wait for {:?}: {error}", options.command);
            Err(error.into())
        }
    }
}

/// Change the current working directory to the specified directory, creating
/// that directory if requested.
fn ensure_directory(path: &Path, create: bool) -> Result<()> {
    if create {
        create_dir_all(path)
            .inspect_err(|error| eprintln!("Could not create directory {path:?}: {error}"))?
    }
    set_current_dir(path)
        .inspect_err(|error| eprintln!("Could not change directory to {path:?}: {error}"))?;
    Ok(())
}

/// Create a temporary directory and change the current working directory to it,
/// creating intermediate directories if requested.
fn ensure_temporary_directory(path: &Path, create: bool) -> Result<TempDir> {
    // The methods `from_os_str` and `to_os_str` below come from
    // `bstr::ByteSlice`. This DTRT on UNIX and Windows.
    let (directory, builder): (_, TempBuilder) = match path.file_name() {
        None => (Some(path), TempBuilder::new()),
        Some(name) => match <[u8]>::from_os_str(name) {
            None => Err(Error::Utf8Invalid(name.to_owned()))?,
            Some(name) => match bytes_regex_captures!(r"^(.*?)(X+)(.*?)$", name) {
                None => (Some(path), TempBuilder::new()),
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
        create_dir_all(directory)
            .inspect_err(|error| eprintln!("Could not create directory {directory:?}: {error}"))?
    }
    let tempdir = if let Some(directory) = directory {
        builder.tempdir_in(directory).inspect_err(|error| {
            eprintln!("Could not create temporary directory in {directory:?}: {error}")
        })?
    } else {
        let directory = env::temp_dir();
        builder.tempdir_in(&directory).inspect_err(|error| {
            eprintln!("Could not create temporary directory in {directory:?}: {error}")
        })?
    };
    set_current_dir(&tempdir)
        .inspect_err(|error| eprintln!("Could not change directory to {tempdir:?}: {error}"))?;
    Ok(tempdir)
}

/// Squash an empty path to `None`.
fn squash_empty_path(path: &Path) -> Option<&Path> {
    if path.iter().next().is_some() {
        Some(path)
    } else {
        None
    }
}
