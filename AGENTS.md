# Agent guide

nitidus is a terminal mail client. It runs Himalaya 3's own commands in-process and reads their
`--json` output (`src/mail.rs`), on a worker thread so the screen never waits on the network
(`src/worker.rs`). The screen is a set of components that own their state and draw themselves
(`src/ui/`), and `src/app.rs` routes keys and worker answers between them. `nitidus demo` runs
against fake accounts and a local SMTP sink, so nothing needs a real login. Run `just ci` before
handing off a change; `just fmt` applies nightly rustfmt and rumdl.

Before starting, read the [roadmap](docs/roadmap.md) for the item you are working on and the
[decisions](docs/decisions/README.md) behind the design. Builds need
`CARGO_RESOLVER_INCOMPATIBLE_PUBLISH_AGE=allow` until early December 2026
([decision 4](docs/decisions/0004-crate-age-override.md)).

Read the convention that applies to the task:

- [Code](docs/conventions/code.md): layout, ordering, imports, and function and module size.
- [UI](docs/conventions/ui.md): look, color, components, and how screens are checked.
- [Mail](docs/conventions/mail.md): how nitidus talks to Himalaya and treats mail as input.
- [Testing](docs/conventions/testing.md): where tests go and what they may touch.
- [Docs](docs/conventions/docs.md): what comments and rustdoc should say.
- [Commits](docs/conventions/commits.md): how changes are shaped and described.
- [Dependencies](docs/conventions/dependencies.md): versions, licenses, and tool configuration.
- [Writing](docs/conventions/writing.md): prose in Markdown, commit bodies, and PRs.

## Working here

- Make each independent review unit its own change.
- GitHub Actions are locked in `.github/workflows/actions.lock`. After changing any `uses:`, run
  `gh actions-lock` and commit both files. See [GitHub Actions](docs/github-actions.md). Never edit
  or delete the lockfile by hand.
- Fix warnings at their source. Suppress a lint only with `#[expect(lint, reason = "...")]`.
- Show visible changes as recordings: add or update a tape in `tapes/`, run `just tape <name>`, and
  share the GIF or MP4 from `target/screens/`. Never commit the output.
- Delete a roadmap item in the change that lands it, and add any bug found along the way. Keep
  status and process notes out of the repository.
- Record a design choice that would surprise a newcomer as a decision in `docs/decisions/`.
- When feedback establishes a reusable rule, add it to the convention file that owns it, with its
  reason. Keep this file as the map.
- Local, machine-specific instructions may live in an ignored `AGENTS.override.md`. Read it if it
  exists.
