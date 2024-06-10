use std::{ffi::OsString, io};

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("No command specified and SHELL not set")]
    NoCommand,
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
            Error::NoCommand => 1,
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
