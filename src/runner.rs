use std::{env, process::Command};
use std::{fs, path::Path};

use bstr::ByteSlice;
use lazy_regex::bytes_regex_captures;
use tempfile::{Builder as TempBuilder, TempDir};

#[cfg(unix)]
use nix::sys::signal::{signal, SigHandler, Signal};

use crate::error;
use crate::options;

type Result<T> = std::result::Result<T, error::Error>;

/// The signals that this program will ignore. They'll be reenabled for child
/// processes so that they can receive them as usual – and be terminated most
/// likely – after which this program can clean up.
#[cfg(unix)]
const TERMINATION_SIGNALS: &[Signal] = &[Signal::SIGINT, Signal::SIGQUIT, Signal::SIGTERM];

/// Execute the command in the specified directory. Works on any platform.
pub(crate) fn run(options: options::Options) -> Result<i32> {
    // Ignore signals so that we're not terminated by them. We'll reenable them
    // later on in the child process, just before exec'ing.
    #[cfg(unix)]
    for sig in TERMINATION_SIGNALS {
        unsafe {
            signal(*sig, SigHandler::SigIgn)?;
        }
    }

    // Origin/whence directory.
    let whence_dir = env::current_dir().inspect_err(|error| {
        eprintln!("Could not determine current directory: {error}");
    })?;

    // Change to the requested directory before executing the command. We keep
    // the guard value around because it might be a temporary directory that
    // deletes itself on [`Drop`].
    let _guard = if options.temporary {
        ensure_temporary_directory(&options.directory, options.create).map(Some)?
    } else {
        ensure_directory(&options.directory, options.create).map(|_| None)?
    };

    // Prepare the command.
    let executable = options.command.first().ok_or_else(|| {
        eprintln!("No command specified and SHELL not set.");
        error::Error::NoCommand
    })?;
    let arguments = &options.command[1..];
    let mut command = Command::new(executable);
    command.args(arguments).env("WHENCE", &whence_dir);

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
        eprintln!("Could not execute {:?}: {error}", executable);
    })?;

    match child.wait() {
        Ok(status) => match status.code() {
            Some(code) => Ok(code),
            None => {
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    match status.signal() {
                        Some(signo) => Err(error::Error::TerminatedBySignal(signo)),
                        None => unreachable!("No exit code or signal"),
                    }
                }
                #[cfg(not(unix))]
                {
                    // We're not on UNIX and we do not know a signal number –
                    // indeed, that concept may not hold here – so we go with 2,
                    // which corresponds to SIGINT on UNIX 🤷
                    Err(error::Error::TerminatedBySignal(2))
                }
            }
        },
        Err(error) => {
            // Not entirely sure how we might get here.
            eprintln!("Could not wait for {:?}: {error}", executable);
            Err(error.into())
        }
    }
}

/// Change the current working directory to the specified directory, creating
/// that directory if requested.
fn ensure_directory(path: &Path, create: bool) -> Result<()> {
    if create {
        fs::create_dir_all(path)
            .inspect_err(|error| eprintln!("Could not create directory {path:?}: {error}"))?
    }
    env::set_current_dir(path)
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
            None => Err(error::Error::Utf8Invalid(name.to_owned()))?,
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
        fs::create_dir_all(directory)
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
    env::set_current_dir(&tempdir)
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
