# GIX

Git tui tool.

⚠️ This is a work in progress and might wreck terminal raw formatting for current session.

## Install

```sh
cargo install --git https://github.com/ludvighe/gix.git
```

## Help

```
$ gix --help
Git tui tool

Usage: gix [OPTIONS]

Options:
  -d, --directory <DIRECTORY>            Path to repository [default: .]
  -s, --summary-length <SUMMARY_LENGTH>  Latest commit summary max length [default: 72]
  -D, --debug                            Render debug info
  -h, --help                             Print help
  -V, --version                          Print version
```
