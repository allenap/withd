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
        help = "Create a temporary directory within the specified directory.",
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
