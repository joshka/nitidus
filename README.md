# nitidus

A small TUI email client written in Rust.

## Why the name?

This app is a front end to the CLI mail client [Himalaya], and is based on a Rust TUI crate named
[Ratatui]. *Rattus Nitidus* is the scientific name for the [Himalayan field rat].

[Himalaya]: https://pimalaya.org/himalaya/
[Ratatui]: https://crates.io/crates/ratatui
[Himalayan field rat]: https://en.wikipedia.org/wiki/Himalayan_field_rat

## Running

nitidus uses [Himalaya] 3 as a library, so it reads the same config as the `himalaya` CLI
(`~/.config/himalaya/config.toml`) and supports every backend Himalaya does (IMAP, JMAP, Gmail,
Microsoft Graph, Maildir, mbox, pimdir).

```shell
nitidus                  # default account
nitidus -a work -m Archive
nitidus demo             # a fake Maildir account, no login needed
```

## What it does

- Every backend Himalaya supports, configured once in Himalaya's config; switch accounts with `A`.
- Mailboxes: browse, create, rename, delete. The envelope list pages through the whole mailbox,
  groups by date or by thread (`T`), and picks up new mail every 30 seconds.
- Reading: plain text and HTML, quoted text set off, attachments to open or save, a link picker, the
  raw source, and opening the HTML part in a browser.
- Triage: read, flag, archive, delete (to the trash; asks first inside it), move or copy through a
  mailbox picker, all on one message or on several marked ones.
- Writing: compose, reply, reply all, forward and reopen drafts in `$EDITOR`, then review, attach
  files, and send or save. Sending goes through Himalaya (SMTP, JMAP, Gmail, Graph).
- Themes: dark and light (`--theme`), with text contrast checked by a test.

## Keys

| Keys                           | Action                                                                 |
| ------------------------------ | ---------------------------------------------------------------------- |
| `j` `k` `g` `G`, `tab` `h` `l` | move, switch pane                                                      |
| `enter`, `esc`                 | open, back / clear marks / clear search                                |
| `s` `f`                        | toggle read / flag                                                     |
| `a` `d` `m` `y`                | archive, delete, move to…, copy to…                                    |
| `x` (or `space`), `V`          | mark, mark all                                                         |
| `c`, `r` `R`, `F`, `e`         | compose, reply / reply all, forward, edit a draft                      |
| `o` `S`                        | open / save attachments                                                |
| `L` `B` `H`                    | links, open HTML in a browser, message source                          |
| `/`, `ctrl-r`, `T`             | search (`from:ada is:unread has:attachment date:2d`), refresh, threads |
| `n` `R` `D` (mailbox pane)     | new, rename, delete mailbox                                            |
| `A`, `?`, `q`                  | switch account, help, quit                                             |

The composer opens your `$VISUAL` / `$EDITOR` on a plain file: headers (`To:`, `Cc:`, `Bcc:`,
`Subject:`, and `Attach:` lines naming files), a blank line, then the body. Back in nitidus you
review it, then `s` sends, `d` saves a draft, `a` attaches, `e` edits again, `x` discards.

## The demo

`nitidus demo` writes two fake Maildir accounts to a temporary directory and starts an SMTP server
on localhost that accepts everything. Mail sent to the demo address (`rae@nitidus.test`) comes back
to its inbox, so the whole send path runs with no real server.

## Screenshots

Recorded with [betamax](https://github.com/joshka/betamax) from `tapes/*.tape` against
`nitidus demo`. Output goes to `target/screens/` (not committed):

```shell
cargo build && for t in tapes/*.tape; do betamax run "$t"; done
```
