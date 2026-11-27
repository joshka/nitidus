# 2. pimdir through Himalaya

pimdir is a local SQLite store a sync engine (Neverest) fills from a server. nitidus treats it as
one more Himalaya backend: point the account at `pimdir.root` and the same code paths work.

## What was checked

`nitidus demo --pimdir` seeds a store from the demo mail as an `imap` source would, and
`src/mail/pimdir_tests.rs` runs against it. Listing, paging, search, threads, reading, attachments,
templates, flags, move, delete, send and drafts all work. Writes come back with `queued: true` and
show in the store at once; Neverest applies them on its next sync.

Creating a mailbox fails in the seeded store, because its fake source declares no capabilities. A
store Neverest fills declares them.

## Consequences

- Never claim a queued write is done: the status bar says "queued for sync", and a queued send is
  "Queued to send on the next sync".
- pimdir names mailboxes `imap/Archive`; show `Mailbox::label`, the last segment.
- Neverest syncs only from servers (IMAP today), never from a local Maildir, and `neverest sync` is
  one-shot: run it under a timer, and with `--queue-only` to push writes quickly. Wiring that into
  nitidus is on the roadmap; it needs a real IMAP account to test.

## Rejected

- io-pimdir's own API: it is stable (1.0), but pimdir-only, and nitidus would drive sync itself.
