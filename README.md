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

## License

[GNU General Public License 3.0](https://www.gnu.org/licenses/gpl-3.0.html) (or
later). See [LICENSE](LICENSE).
