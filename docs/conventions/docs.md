# Documentation conventions

- Document every public item. The compiler enforces this.
- Document private items too, naming their intent or what they expect: preconditions, invariants,
  ordering, or ownership. Clippy enforces this with `missing_docs_in_private_items`.
- A doc must tell the reader something the name and signature don't. "Parses the size" fails.
  "Rejects values above `maximum` so callers can't overflow the unit multiplier" passes.
- Start with one sentence of about 15 words saying what the item does.
- Describe what the code does now. Rationale goes beside the code it explains.
- Start every module with a `//!` doc saying what it's for and any rule it enforces.
- Let `main.rs` teach the tool: what it's for and how its stages fit together.
- Document failures under `# Errors` and panics under `# Panics`, where they apply.
- Use one empty doc-comment line after a heading and between paragraphs, never two.
- Prefer doctests that run. Use `no_run` when an example needs the network, and say why if one must
  be `ignore`.
- Give each fact one home and link to it: contracts in rustdoc, the pipeline in `main.rs`, and
  change context in the commit or PR.
- Markdown prose follows the [writing conventions](writing.md).
