# Roadmap

What is known to be wrong and what comes next, each item sized for one change. Delete an item in the
change that lands it; the history records what was done.

## Known bugs

1. A request that fails outright leaves its optimistic change on screen (a flag, a move, a delete);
   only a partial failure reloads the list. Fix: on `Response::Failed` for a write, reload the list,
   and say so.
1. Reopening a saved draft (`e` in Drafts) reads the raw source, so a quoted-printable, base64 or
   multipart draft comes back encoded and loses its attachments. Fix: read it with `message read`
   (decoded) and carry its attachments, or ask Himalaya for a template from it.
1. `G` goes to the last loaded envelope, not the end of the mailbox; a search only covers the pages
   fetched.
1. The new-mail check only adds messages. Deletions, moves and flag changes made elsewhere show only
   after `ctrl-r`.
1. An editor that returns at once (`code` without `-w`) leaves the draft unchanged with no warning.

## Next

Roughly in order; each is one change with tests and, when visible, a tape.

1. Undo for archive, delete and move (`u`), from the receipts' new ids where the backend gives them,
   else by moving back.
1. Unread counts for every mailbox, through `mailbox list --counts` where the backend answers
   cheaply (IMAP, JMAP, Graph); none for Maildir.
1. Recipient completion in the composer from a contacts command (the `aerc` format:
   `address<TAB>name` per line), configured under a `[tui]` table.
1. Compose polish: show the signature in the review screen, remove an attachment, keep an autosave
   of the editor file until the draft is sent or discarded.
1. Attachments: open or save one chosen attachment rather than all.
1. Reading: search within a message; a conversation view of a whole thread; richer HTML (no leftover
   `**` and `[1]`).
1. Instant new mail through IMAP IDLE instead of polling.
1. pimdir: run `neverest sync --queue-only` after writes, and mark queued messages in the list
   ([decision 2](decisions/0002-pimdir-through-himalaya.md)).
1. Mailboxes: show nested folders as a tree; create a mailbox from the move picker.
1. Configuration: a `[tui]` table for key bindings and the theme; detect a light terminal with an
   OSC 11 query; a sort order.
1. Larger: a unified inbox across accounts, search across mailboxes, desktop notifications, mouse
   support.
