# Testing conventions

- Test mail behavior against the demo account (`demo::create`), through `MailClient`, the same path
  the UI uses. Test component logic (ordering, threading, marks) on the component, without a
  terminal.
- Keep a test linear: set up, act once, compare whole values. It should read as documentation.
- Name a test for the behavior it proves, not the function it calls.
- Assert on values rather than booleans, so a failure shows what happened.
- Add a regression test with every bug fix.
- Keep tests deterministic, off the network (the SMTP sink listens on localhost only), and out of
  the user's config.
- Check visible changes with a tape as [UI](ui.md) describes; a recording is review evidence, not a
  test.
- Run tests with `just test`.
