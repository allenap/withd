# `withd`

`withd` is a simple command-line tool that allows you to run a command with a
different working directory _without_ affecting the current shell's working
directory.

## Why is this useful?

Many commands – such as `git`, `npm`, and `cargo` – require you to run them from
a specific directory. This can be done by `cd`ing into the directory:

```bash
cd /path/to/repo
git status
cd -
```

or by using a subshell to isolate the change:

```bash
( cd /path/to/repo && git status )
```

The first is cumbersome. The latter can be confusing when also trying to work
with shell variables in a script, for example, since the subshell cannot
propagate changes to the parent shell. It's also easy to forget.

Then there's `CDPATH`. If this is set in your shell, `cd`'s behaviour changes
and you might end up in a different directory than you expected. I've seen this
be a source of confusion – and a disruptive and very difficult to diagnose bug.

`withd` does not have these problems. It's simple and predictable.

## `withd --help`

```shellsession
$ withd --help
Run a command in another directory.

Usage: withd [OPTIONS] <DIRECTORY> <COMMAND> [ARGS]...

Arguments:
  <DIRECTORY>
          The directory in which to execute the command.

  <COMMAND>
          The command to execute.

  [ARGS]...
          The arguments to pass to the command.

Options:
  -c, --create
          Create the directory if it does not exist.

  -t, --temporary
          Create a temporary directory within DIRECTORY. This temporary directory will be deleted when the command completes. Note that this option modifies slightly how the DIRECTORY argument is used. For example:

          - `withd -tc foo/bar.XXXX.baz …` will create the directory `foo` (and will not remove it later on) and a temporary directory inside it called `bar.1234.baz` (where the 1234 is random).

          - `withd -tc foo …` will create `foo`, as above, and a temporary directory inside it named `.tmp123456` (again, where 123456 is random).

          - `withd -t foo …` will create a temporary directory named `.tmp123456` (again, where 123456 is random) in `foo`, but assumes that `foo` already exists.

          - `withd -t foo.XXXX.bar …` will create a temporary directory named `foo.1234.bar` in the system's temporary directory, e.g. $TMPDIR.

          - `withd -t "" …` will create a temporary directory named `.tmp123456` in the system's temporary directory, e.g. $TMPDIR.

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Execute a command in a specific directory.
```

## Installation

For now, use [Cargo](https://doc.rust-lang.org/cargo/):

```bash
cargo install withd
```

## Usage

```bash
withd /path/to/repo git status
```

To create the directory:

```bash
withd -c /some/where echo "Hello, world!"
```

(`-c` is short for `--create`.)

## Making a release

1. Bump version in [`Cargo.toml`](Cargo.toml).
2. Paste updated `--help` output into [`README.md`](README.md) (this file; see
   near the top). On macOS the command `cargo run -- --help | pbcopy` is
   helpful. **Note** that `--help` output is not the same as `-h` output: it's
   more verbose and that's actually what we want here.
3. Build **and** test. The latter on its own does do a build, but a test build
   can hide warnings about dead code, so do both.
   - With default features: `cargo build && cargo test`
   - Without: `cargo build --no-default-features && cargo test --no-default-features`
4. Commit with message "Bump version to `$VERSION`."
5. Tag with "v`$VERSION`", e.g. `git tag v1.0.10`.
6. Push: `git push && git push --tags`.
7. Publish: `cargo publish`.

## License

[GNU General Public License 3.0](https://www.gnu.org/licenses/gpl-3.0.html) (or
later). See [LICENSE](LICENSE).
