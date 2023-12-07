# nitidus

A small TUI email client written in Rust.

## Why the name?

This app is a front end to the CLI mail client [Himalaya], and is based on a Rust TUI crate named
[Ratatui]. *Rattus Nitidus* is the scientific name for the [Himalayan field rat].

[Himalaya]: https://pimalaya.org/himalaya/
[Ratatui]: https://crates.io/crates/ratatui
[Himalayan field rat]: https://en.wikipedia.org/wiki/Himalayan_field_rat

## Running

This is pretty much just a PoC right now, so install himalaya, run it once to generate a config and
then run this. Check the command line params with `--help`

```text
A TUI email client

Usage: nitidus [OPTIONS]

Options:
  -c, --config <FILE>           A path to a himalaya configuration file
  -a, --account-name <ACCOUNT>  The name of the account to use from the configuration file
  -f, --folder <FOLDER>         The mail folder to open
  -h, --help                    Print help
  -V, --version                 Print version
```

![demo](https://github.com/joshka/nitidus/assets/381361/889b752f-b5e2-4393-bb19-071190c9fc25)
