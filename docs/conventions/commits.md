# Commit conventions

- Make each commit one coherent, reasonably sized step that builds and passes its tests alone.
- Land tests and docs with the behavior they cover, with no unused code waiting for a later commit.
- Write an imperative subject of about 50 characters, without a conventional-commit prefix.
- Use the body to say why, wrapped at 72 columns.
- Keep unrelated cleanup out of the change.
- Write PR descriptions as the [writing conventions](writing.md) describe.
- Separate structural changes (moves, renames, refactors) from behavior changes. An "and" in the
  subject is a sign the change holds two.
- Before changing messy behavior, land tests that pin what it does now.
- Don't commit large binaries, such as recordings and other captures, because history keeps every
  version and each clone pays for it. Commit the text that makes them, such as tapes, and write the
  output to an ignored folder.
