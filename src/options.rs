use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use clap::{command, Parser};

#[derive(Parser)]
#[command(
    author, version, about, long_about = None,
    after_help = "Execute a command in a specific directory.",
)]
pub(crate) struct Options {
    #[arg(
        help = "The directory in which to execute the command.",
        value_parser = EmptyPathBufValueParser
    )]
    pub(crate) directory: PathBuf,

    #[arg(
        short,
        long,
        help = "Create the directory if it does not exist.",
        default_value_t = false
    )]
    pub(crate) create: bool,

    #[arg(
        short,
        long,
        help = concat!(
            "Create a temporary directory within the directory specified by ",
            "-c/--create."
        ),
        long_help = concat!(
            "Create a temporary directory within the directory specified by ",
            "-c/--create. This temporary directory will be deleted when the ",
            "command completes. This option modifies how the -c/--create ",
            "option behaves: when the last component of the directory given ",
            "includes any number of 'X's, they will be replaced with a unique ",
            "string of the same length.",
            "\n\n",
            "For example:",
            "\n\n",
            "- `withd -tc foo/bar.XXXX.baz` will create the directory `foo` ",
            "(and will not remove it later on) and a temporary directory ",
            "inside it called `bar.1234.baz` (where the 1234 is random).",
            "\n\n",
            "- `withd -tc foo` will create `foo` as above, and a temporary ",
            "directory named `.tmp123456` (again, where 123456 is random).",
            "\n\n",
            "- `withd -tc foo.XXXX.bar` will create a temporary directory ",
            "named `foo.1234.bar` in the system's temporary directory, e.g. ",
            "$TMPDIR",
            "\n\n",
            "- `withd -tc \"\"` will create a temporary directory named ",
            "`.tmp123456` in the system's temporary directory, e.g. $TMPDIR",
        ),
        default_value_t = false
    )]
    pub(crate) temporary: bool,

    #[arg(help = "The command to execute.")]
    pub(crate) command: OsString,

    #[arg(
        help = "The arguments to pass to the command.",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub(crate) args: Vec<OsString>,
}

// -----------------------------------------------------------------------------

#[derive(Copy, Clone)]
/// [`clap::builder::PathBufValueParser`] has a limitation: it treat empty paths
/// as errors. This parser allows for empty paths.
pub struct EmptyPathBufValueParser;

impl clap::builder::TypedValueParser for EmptyPathBufValueParser {
    type Value = PathBuf;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> std::result::Result<Self::Value, clap::Error> {
        Ok(value.to_owned().into())
    }

    fn parse(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: OsString,
    ) -> std::result::Result<Self::Value, clap::Error> {
        Ok(value.into())
    }
}
