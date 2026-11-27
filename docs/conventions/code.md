# Code conventions

## What matters most

These hold everywhere, ahead of any project detail below.

- **Small functions.** 25 lines or fewer, split by step. A body reads as the algorithm: named steps
  called in order. `too_many_lines` enforces the limit.
- **No large match blocks.** An arm longer than a line or two calls a function named for its case,
  so the match reads as a table of what happens.
- **Explanatory variables.** When rustfmt wraps an expression awkwardly, or a value needs a name to
  be understood, bind it to a well-named local first.
- **One way to fail per line.** Bind each fallible call's result to a named local instead of
  chaining several `?` in one expression, so an error points at one call.
- **Data looks like data.** Key bindings, hints, help text and fixtures are tables (`const` arrays
  of small structs or tuples), not code that builds them step by step. A `match` over a list of
  cases dispatches with one line per arm to a function named for the case, so it reads as a table of
  what happens.
- **One concept per type.** A type owns one thing and its impls stay at one level of abstraction;
  payloads shared by several variants become their own types (`MessageRef`, `Selection`) instead of
  repeated fields. Prefer an enum to a `bool` parameter or a cluster of `bool` fields.
- **Docs on every item.** Public and private, each saying something the name and signature don't.
  `missing_docs` and `missing_docs_in_private_items` enforce it; [docs](docs.md) says what to write.

## Everything else

- Lints in `Cargo.toml` back these conventions, and `just ci` denies warnings. Fix the code rather
  than the lint; if a lint is wrong for one item, `#[expect]` it with a reason. The terminal belongs
  to the UI: only `main` prints, and it expects `print_stdout` where it does.
- Order items for reading: a type with its impls, callers before callees, and public before private
  when those don't decide it. These are defaults; choose the order that reduces jumping around.
- Use `foo.rs` with a `foo/` directory, never `mod.rs`. Clippy enforces this.
- Group imports by module, which rustfmt enforces. Use a qualified path when it reads better.
- Keep functions to 25 lines or fewer. Split a long function by step, not at an arbitrary line.
  Clippy enforces this with `too_many_lines`; the threshold is in `clippy.toml`.
- Suppress a lint with `#[expect(lint, reason = "...")]`, never `#[allow]`, except on generated
  code.
- Don't take `bool` parameters, which read as a bare `true` at the call site. Split the function, or
  pass the value the decision comes from. Clippy enforces this.
- Keep tool configuration in `.config/` where the tool supports it. `rustfmt.toml` and `clippy.toml`
  stay at the root because those tools can't find them elsewhere without help, and `cargo-deny`
  needs an explicit `--config`.
- Put a blank line before a doc comment, including on struct fields and enum variants, so long lists
  stay easy to scan.
- Give each module one concept, named for what it owns. No `utils.rs` or `types.rs` buckets.
- Cut modules by feature, keeping a feature's types, helpers, and tests together, not by technical
  layer. Keep a helper used once next to its caller.
- Keep files readable in one pass, around 200 to 400 lines including tests. Past that, check whether
  the file owns more than one concept; don't split one clear algorithm to hit a number.
- Make a function body read as the algorithm: name the steps as functions and call them in order.
  When a block needs a comment to say what it does, extract it and let the comment become its doc.
- Write bodies as short paragraphs, one step each: setup, the operation, then the result. Put a
  blank line around a multi-line `if`, `match`, or loop unless it consumes the line just above.
- Bind a multi-line receiver or struct literal to a local before chaining on it or passing it.
- Keep each function at one level of abstraction. Don't mix calls to named steps with the low-level
  detail of one of them; move the detail into its own function.
- Keep match arms short. When an arm needs more than a line or two, call a function named for that
  case, so the match reads as a table of what happens.
- Push `if`s up and `for`s down: decide at the caller and keep the callee unconditional, and make a
  function work on a whole batch rather than call it once per item in a loop.
- Allow one way to fail per line. Bind the result of each fallible call to a named local, rather
  than chaining several `?` or `.await?` steps in one expression.
- Read how rustfmt wraps the code. When it breaks an expression awkwardly, bind the parts to named
  locals or extract a function, so the code reads without the visual noise of the wrapping.
- Use iterators for transformations and plain statements for I/O and mutation, so side effects stay
  visible.
- Pass a function what it needs, not a broad context object.
- Use enums for states and closed choices, not `bool`s or strings. Parse outside strings into domain
  types at the boundary.
- Make tuning numbers named `const`s with a doc saying what they control.
- Prefer boring, direct code. Don't add a registry, framework, or config option for a small closed
  choice, and don't optimize until a measurement says where the time goes.
- No `unwrap` or `expect` outside tests unless the invariant is local and stated beside it. Clippy
  warns on `unwrap_used`.
- Fix a warning at its source rather than suppressing it.
- Keep computation pure and push I/O, time, and mutation to the edges, so the core tests without
  fakes.
- Introduce a seam (trait, generic) only for variation that exists now, such as a real backend and a
  test fake. Two similar blocks that change for different reasons stay separate.
