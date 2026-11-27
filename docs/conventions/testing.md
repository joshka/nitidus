# Testing conventions

- Every module with behavior has tests beside it, in a `tests` module at the bottom. Test mail
  behavior against the demo account (`demo::create`) through `MailClient`, the same path the UI
  uses; test component logic on the component, without a terminal.
- Show what a screen looks like, don't assert cells. Render a component into a buffer with
  `ui::test_support::render` and compare its text with an insta snapshot. Keep a component's
  snapshot inline (`assert_snapshot!(screen, @"…")`) so the picture sits beside the test; put a
  whole-screen snapshot in a `.snap` file, where it doesn't push the test past the function limit.
- Snapshot only what a test controls. Use the fixtures in `mail::fixtures`, dated in 2020, and
  `demo::create_at` with a fixed time; mask what the local time zone shifts with insta filters.
- Write a table of cases as an `rstest` with named `#[case::…]`s, so a failure names the case.
- Keep a test linear: set up, act once, compare whole values. It should read as documentation.
- Name a test for the behavior it proves, not the function it calls.
- Assert on values rather than booleans, so a failure shows what happened.
- Add a regression test with every bug fix; a snapshot that shows the bug is the quickest.
- Keep tests deterministic, off the network (the SMTP sink listens on localhost only), and out of
  the user's config. Use `indoc!` for multi-line text.
- Check visible changes with a tape as [UI](ui.md) describes; a recording is review evidence, not a
  test.
- Run tests with `just test`; review changed snapshots with `just snapshots`.
