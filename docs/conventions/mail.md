# Mail conventions

- Reach mail only through Himalaya's commands, run in-process with an argv nitidus builds, and
  decode their `--json` output. That output is Himalaya's versioned contract; its Rust types are
  not, so nitidus never imports them.
- Build an argv one value per element and put `--` before ids and paths, so no value is read as a
  flag or split like a shell line. Pass message bodies through a temporary file, never inline.
- Let Himalaya build and send messages. Compose, reply, and forward start from its JSON template and
  go back through its flags, with `--posting-style none` once the user has laid out the quote.
  nitidus writes no MIME of its own.
- Treat a partial outcome as a result: act on what succeeded and report each item that failed.
- A backend may queue writes for a sync engine (pimdir). Show the change at once and say it is
  queued; never claim a queued send was sent.
- Treat message content as untrusted. Render it as text, never interpret terminal escapes from it,
  and open links or attachments only on an explicit key.
- Keep the demo honest: `nitidus demo` uses the same code paths as a real account, with fake Maildir
  (or pimdir) data and a local SMTP sink, so recordings and tests show real behavior.
