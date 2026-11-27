# UI conventions

- Tell panels apart by background tint, not borders. Box-drawing lines cost a row or column each and
  make a dense screen busy; a tint separates without using space.
- Show selection, marks, and focus with a filled background and a one-cell accent edge, never by
  reversing colors or adding brackets.
- Name colors by role in `src/ui/theme.rs` (`dim`, `faint`, `accent`, `selected`). Components never
  use a raw color, so a theme is one table.
- Keep text readable: body text and `dim` at 4.5:1 or more against every surface they sit on,
  `faint` at 3:1 and never on a selected row. The theme test checks both palettes; a new color or
  surface goes into it.
- Never show state by color alone. Pair it with a glyph or a word (`●` unread, `★` flagged, `✓`
  marked, "queued for sync").
- Keys live in one table, `src/app/keymap.rs`: dispatch, the status bar's hints and the help sheet
  all read it, so a binding changes in one place. Overlays (pickers, prompts, questions) handle
  their own keys and answer with an `Outcome`.
- Each component owns its state and draws itself through `Widget`. `App` routes keys and worker
  answers; it doesn't draw rows or know a component's layout.
- Never block the event loop on the backend. Send the request to the worker, show what is running in
  the top bar, and fold the answer in when it arrives. Update the screen first when the outcome is
  predictable (a flag, a move), and reload when the receipt names a failure.
- Say what happened in the status bar, from the receipt: what was done, how many, and what failed.
- Lay out for 110 columns and up as three panes, and for narrower terminals as one pane at a time.
- Check a visible change by recording it with betamax from `nitidus demo`, never from a real
  account. Keep tapes in `tapes/` and their output in the ignored `target/screens/`.
