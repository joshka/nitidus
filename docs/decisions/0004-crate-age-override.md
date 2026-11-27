# 4. New crates despite the age rule

The machine's Cargo config refuses crates published in the last seven days. Himalaya 3.0.0 and the
Pimalaya crates under it went 1.0 on 2026-11-26, so every build sets:

```sh
export CARGO_RESOLVER_INCOMPATIBLE_PUBLISH_AGE=allow
```

## Why

The point of the rewrite is the current Pimalaya stack; the older releases are a different
architecture. The override is a conscious, temporary exception, not a habit: other new dependencies
are pinned to a version older than seven days (rstest 0.26 rather than 0.29).

## Consequences

- CI needs the same variable until the window passes (around 2026-12-04), then drop it here and in
  any workflow.
- `cargo deny` allows MPL-2.0 (option-ext, through Himalaya's `dirs`) and ignores the chrono
  "unmaintained" advisory, both with reasons in `.config/deny.toml`.
