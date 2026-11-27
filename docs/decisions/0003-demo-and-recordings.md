# 3. Demo and recordings

## The demo

`nitidus demo` writes two Maildir accounts (`demo` and `work`) to a temporary directory and starts
an SMTP server on localhost that accepts everything; mail to `rae@nitidus.test` comes back to the
demo inbox. `--pimdir` seeds a pimdir store instead. `--dir <path> --setup-only` writes the accounts
and prints the config path, to point another client (himalaya, himalaya-tui) at the same mail.

The demo uses the same code paths as a real account, so tests (`demo::create`, `demo::create_at`
with a fixed time) and recordings show real behavior. Never test or record against a real account.

## Recordings

Tapes live in `tapes/` and run with betamax (`just tape <name>`, `just tapes`); they write to the
ignored `target/screens/`. Never commit the output: history keeps every version of a binary and
every clone pays for it.

- Share progress as moving pictures while working, linked in the conversation as they are made: MP4
  or WebM for long or captioned runs, GIF for short ones.
- `tapes/tour.tape` uses `Caption` to explain each step; keep it covering every feature.
- Tapes set `EDITOR="vim -Nu NONE"` so composing can be scripted, and `NITIDUS_DOWNLOADS` so saved
  attachments stay in `target/`.
- `tapes/himalaya-tui.tape` records himalaya-tui on the same demo mail; it needs `HIMALAYA_TUI` set
  to a built binary.
