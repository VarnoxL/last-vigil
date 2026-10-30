## Session handoffs

Handoffs live in `docs/handoff/`, one file per session, named `YYYY-MM-DD-<slug>.md` so they sort by date.

- **Start of session:** read the newest handoff and continue from its "Next" section.
- **End of session:** when the user asks for a handoff, write a new file with these sections: Done, State (branch, last commit, what builds and passes), Next, and Notes (decisions, gotchas, user preferences). Don't edit old handoffs.
- The user is learning Rust in `sim/`. Guide them with explanations, hints and reviews instead of writing the code, unless they ask for it.

## Agent skills

### Issue tracker

Issues and specs live as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default labels: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

One glossary for the whole repo: `GLOSSARY.md` and `docs/adr/` at the root. See `docs/agents/domain.md`.
