# Tools come from .config/mise.toml (`mise install`); rustfmt needs a nightly toolchain.

default:
    @just --list

# Format Rust (nightly rustfmt, for its unstable options) and Markdown.
fmt:
    cargo +nightly fmt --all
    rumdl fmt .

fmt-check:
    cargo +nightly fmt --all -- --check

clippy:
    cargo clippy --all-targets --all-features --locked -- -D warnings

# nextest doesn't run doctests, so cargo test runs them.
test:
    cargo nextest run --all-features --locked
    cargo test --doc --all-features --locked

# Check that the Rust documentation builds without warnings.
rustdoc:
    RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items --locked

deny:
    cargo deny --all-features --config .config/deny.toml check

machete:
    cargo machete

typos:
    typos --config .config/typos.toml

lint-md:
    rumdl check .

# Run every check that works offline.
ci: fmt-check clippy test rustdoc deny machete typos lint-md

# Run the fake demo account: no login, a local SMTP server that accepts everything.
demo *args:
    cargo run --locked -- demo {{ args }}

# Record every tape to target/screens/ (needs betamax and vim).
tapes: (build)
    for tape in tapes/*.tape; do betamax run "$tape"; done

# Record one tape, e.g. `just tape compose`.
tape name: (build)
    betamax run tapes/{{ name }}.tape

build:
    cargo build --locked
