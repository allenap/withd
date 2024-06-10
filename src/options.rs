use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use clap::{command, Parser, ValueHint};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    author, version, about, long_about = None, max_term_width = 80,
    after_help = concat!(
        "The executed command can use the `WHENCE` environment variable to ",
        "refer back to the directory from whence `withd` was invoked.",
    ),
)]
pub(crate) struct Options {
    #[arg(
        help = "The directory in which to execute the command.",
        value_parser = EmptyPathBufValueParser,
        value_hint = ValueHint::AnyPath,
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
            "Create a temporary directory within DIRECTORY. This temporary ",
            "directory will be deleted when the command completes. Note that ",
            "this option modifies slightly how the DIRECTORY argument is ",
            "used. For example:",
            "\n\n",
            "- `withd -tc foo/bar.XXXX.baz …` will create the directory `foo` ",
            "(and will not remove it later on) and a temporary directory ",
            "inside it called `bar.1234.baz` (where the 1234 is random).",
            "\n\n",
            "- `withd -tc foo …` will create `foo`, as above, and a temporary ",
            "directory inside it named `.tmp123456` (again, where 123456 is ",
            "random).",
            "\n\n",
            "- `withd -t foo …` will create a temporary directory named ",
            "`.tmp123456` (again, where 123456 is random) in `foo`, but ",
            "assumes that `foo` already exists.",
            "\n\n",
            "- `withd -t foo.XXXX.bar …` will create a temporary directory ",
            "named `foo.1234.bar` in the system's temporary directory, e.g. ",
            "$TMPDIR.",
            "\n\n",
            "- `withd -t \"\" …` will create a temporary directory named ",
            "`.tmp123456` in the system's temporary directory, e.g. $TMPDIR.",
        ),
        default_value_t = false
    )]
    pub(crate) temporary: bool,

    #[arg(
        long,
        hide = true,
        help = "Generate shell completions.",
        // exclusive = true
    )]
    pub(crate) completions: Option<Shell>,

    #[arg(
        help = "The command and its arguments.",
        trailing_var_arg = true,
        allow_hyphen_values = true,
        value_hint = ValueHint::CommandWithArguments,
        num_args = 1..,
        env = "SHELL",
    )]
    pub(crate) command: Vec<OsString>,
}

// -----------------------------------------------------------------------------

#[derive(Copy, Clone)]
/// [`clap::builder::PathBufValueParser`] has a limitation: it treat empty paths
/// as errors. This parser allows for empty paths.
struct EmptyPathBufValueParser;

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
