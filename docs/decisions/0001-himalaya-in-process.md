# 1. Himalaya in process

nitidus runs Himalaya's own commands inside its process (`himalaya::cli::Command`, parsed from an
argv nitidus builds) over one `Session`, and decodes their `--json` output into its own types in
`src/mail/`.

## Why

- Himalaya's library promises nothing beyond its command grammar and that JSON, which is versioned.
  Its Rust types change between releases; the JSON doesn't without notice.
- One seam gives every backend Himalaya supports (IMAP, JMAP, Gmail, Graph, Maildir, mbox, pimdir)
  and its sending, MIME building, threading headers and config, with no code of ours.
- The Himalaya CLI and nitidus share one config file.

## Consequences

- Composing starts from Himalaya's JSON template and submits through the same command's flags, with
  `--posting-style none` because the edited body already holds the quote. nitidus writes no MIME.
- Each call parses an argv. Build it one value per element, `--` before ids and paths, and pass
  message bodies through a temporary file.
- himalaya-tui takes the same approach but lags the CLI; don't base designs on its behavior.

## Rejected

- io-* crates directly (io-imap, io-pimdir): more control, but one backend at a time, and nitidus
  would duplicate what Himalaya already wraps.
