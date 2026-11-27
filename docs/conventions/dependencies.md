# Dependency conventions

- Depend on `himalaya` for mail and on io-* crates only where Himalaya's commands can't do the job
  (the demo's pimdir seeding).
- Prefer a maintained crate to writing our own, unless what we need is small or nothing fits.
- Require the lowest version the code honestly works with, without a patch version unless the code
  needs something a specific patch release added. Compatible updates go in `Cargo.lock`, not the
  manifest.
- Turn off default features where that's practical, and enable only the features the code uses.
- If a lint proves more noise than value, turn it off in `Cargo.toml` with a comment saying why.
- Put routine dependency updates in one change, and any update that changes behavior in its own.
- Configure tool rules by name, not ID, where the tool allows it. In `rumdl.toml` write
  `[line-length]`, not `[MD013]`.
- Every check is a `just` recipe, and `just ci` runs them all. Tool versions live in
  `.config/mise.toml`.
- Allow a permissive license in `.config/deny.toml` when a dependency needs it. Ask before allowing
  a copyleft one, even for one crate: the check is there so that a license that spreads to this
  code, or limits how it is shipped, is never taken by accident.
- Set `rust-version` to current stable and raise it as stable moves. Nothing needs an older
  compiler.
