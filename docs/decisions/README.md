# Decisions

Design choices and their reasons. Read the one that applies before changing how nitidus reaches
mail, how it is checked, or how it builds. Add one when a choice would surprise a newcomer.

1. [Himalaya in process](0001-himalaya-in-process.md): mail goes through Himalaya's own commands and
   their JSON, not its Rust types or the io-* crates.
1. [pimdir through Himalaya](0002-pimdir-through-himalaya.md): the local-first store is one more
   backend, synced by Neverest, not a separate design.
1. [Demo and recordings](0003-demo-and-recordings.md): fake accounts and a local SMTP sink; tapes in
   the repository, their output never.
1. [New crates despite the age rule](0004-crate-age-override.md): builds allow the Pimalaya 1.0
   crates while they are younger than seven days.
