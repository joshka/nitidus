# Contributing

nitidus is an experiment, not a serious email client. It was built quickly, largely with AI coding
agents, to try ideas: how a ratatui interface can look (background tints rather than borders,
components that draw themselves), and how far Himalaya's in-process commands go as the whole mail
layer.

The source is shared so the ideas can be read, borrowed and argued with. Contributions aren't being
sought:

- Pull requests will likely be closed without review. Please don't spend time on one.
- Issues are welcome as notes or ideas, but expect no response or timeline.
- Fork it freely; the license allows it.

If something here is useful to you, the better place for most fixes is upstream:
[Himalaya](https://github.com/pimalaya/himalaya) for mail behavior and
[Ratatui](https://github.com/ratatui/ratatui) for the terminal library.

To read or run the code, start with [AGENTS.md](AGENTS.md), which maps the source and links the
conventions, and try `cargo run -- demo`, which needs no mail account.
