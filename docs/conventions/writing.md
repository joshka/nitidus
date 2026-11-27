# Writing conventions

These apply to Markdown docs, commit bodies, and PR descriptions. Rustdoc follows [docs](docs.md) as
well.

- Write for a reader with no chat history. Lead with the result or behavior, then the evidence and
  limits needed to judge it.
- Ground claims in code, tests, history, or measurements. Never invent intent or precise numbers,
  and keep verified facts apart from plans and guesses.
- Use prose for causes and tradeoffs, and lists for steps, checks, and short option sets.
- Name the type, input, effect, or limit instead of describing it vaguely. Keep one term for one
  thing.
- Cut stock phrasing: page narration ("this section covers"), throat-clearing ("it's worth noting"),
  forced contrasts ("not just X, but Y"), vague praise ("powerful", "robust"), and summaries after
  each section.
- Review a draft twice: first for structure and accuracy, then for generated-prose tells. Stop when
  another pass would only swap equivalent words.
- Wrap Markdown at 100 columns and align tables. `just lint-md` checks both.
- Use US spelling. `just typos` checks it.
- Number every ordered-list item `1.`, and give fenced code blocks a language.
- Make docs, commits, and PRs stand alone, without chat history or private notes. Link issues and
  PRs by descriptive title, and summarize what a linked change did.

## Pull requests

- Open with the user need and what now works, in one or two sentences.
- Put independent changes in short bullets, each starting with the useful fact.
- Keep validation compact: what actually ran, its results, and what couldn't run. Say which checks
  were local and which were CI.
- Show the usage near the top when a change adds a feature, then the result.
- For a visible change, include before and after screenshots with a short caption on each.
- Leave out implementation inventories, chronology, and private workspace status.
- Don't hard-wrap PR paragraphs; GitHub renders the line breaks. Pass the body with `--body-file`.
