# Handoff: 2026-11-27, the hearing check's collect phase

## Done
- PR #6 (merged, `f01bda6`): the collect half of the hearing check, wired into `Sim::tick`. Rumors now spread in a real run once something is planted.
  - `pick_claim`: a weighted pick over `known[teller]` by `confidence / (1 + times_told × (1 − influence))`, with one `rng.unit()` draw.
  - `mutate`: only `Hoarder(id)` mutates, with `p = 0.05 + 0.15 × fear`. It swaps the id to an agent in the teller's sight, or to anyone if nobody is in sight, never to the teller or the one already accused. Yes/no propositions draw nothing.
  - `collect_hops`: tellers go in id order. The rng draw order is fixed and documented in `docs/rumor-engine.md`: empty `known` draws nothing, then the speak roll, then the audience (nobody in range ends it, and nothing counts as told), then one listener when below `BROADCAST_INFLUENCE`, then `pick_claim`, then one `mutate` per telling.
  - `Sim::tick` runs `hearing_check` every `CHECK_EVERY` ticks.
  - Tests filled in: `mutation_rate_and_ancestry` and `broadcast_vs_single_hearer`. The wall test is extended, so a speaker behind a wall produces no hop.
- Decisions made after review (`555f571`), now in the doc and the README:
  - **One version per root:** for `Hoarder`, `holds` compares only the variant (`mem::discriminant`), not who is named. A hearer keeps only the first version of a mutated rumor. Before this, mutated variants multiplied to about 90% of the arena. Corroboration still needs the exact same proposition from a different root.
  - **No self-accusation:** `apply_hop` rejects `Hoarder(hearer)`, because the accused knows better. It still counts as told.
  - **Chain length:** chains run 9–11 hops through tellers with influence ≥ 0.676, who don't decay a claim. This is intended.
  - New test: `one_version_per_root_and_never_self`.
- The user wrote the PR. Claude reviewed it on GitHub (inline comments, a plain comment review), checked the fix commit, ran the tests and merged.

## State
- On `main` at `f01bda6`. The `hearing-collect` branch is deleted on GitHub and locally.
- `cargo test`: 18 passed, 1 ignored (`one_sighting_per_watcher`, which needs `Action::Hoard`).
- Clippy flags only nits: three `is_multiple_of`, unused `Sim::plant`, one collapsible `if`, and one `print!` ending in a newline.
- Smoke runs from the PR (not committed): planted claims reach the whole town of 50 in about a day.

## Next
1. `Action::Hoard`, then `record_sightings`. This also lets you finish the wall test's "no sighting" TODO and `one_sighting_per_watcher`.
2. `drift_earned`, then wire it and `record_sightings` into `Sim::tick` (the TODO next to `hearing_check`).
3. `Proposition::bark`.
4. Serde plus the `--plant`/`--dump` CLI flags (ancestry log and JSON export). `Sim::plant` stops being dead code here.
5. `belief_feedback`, last, per the PRD. Then tune the spread speed through `CHECK_EVERY`, the base of `p_speak` and `BROADCAST_INFLUENCE`. Don't tune before that.

## Notes
- Open question: agents who are `Busy(Sleep)` can still speak and hear, and the doc doesn't say whether they should. That matters for family members sharing a home.
- Still open: should the Prophet and Priest start with high loyalty/faith, maybe tied to `prophet_is_fraud`? And stations don't affect behavior yet. Both TODOs are on `assign_stations`.
- `p_speak`'s ×2 checks only `Busy(Socialize)`, not the tile. That's fine, because Socialize only happens at the tavern.
- Workflow: the user writes the Rust and opens a PR, Claude reviews and posts inline comments, the user fixes, and Claude merges with `--merge --delete-branch` once the fix looks good and tests pass.
- No Claude attribution in commits or PRs (user preference).
- `cargo` isn't on PATH in PowerShell; use the Bash tool (`cd sim && cargo test`) or `~\.cargo\bin\cargo.exe`.
- The local branch `readme-schematics` is old and was left alone.
