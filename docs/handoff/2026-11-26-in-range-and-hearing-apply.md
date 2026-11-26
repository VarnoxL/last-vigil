# Handoff: 2026-11-26, in_range and the hearing check's apply phase

## Done
- PR #4 (merged, `15dc317`): `in_range(world, a, b, range)` checks distance, then same room, then no wall on a Bresenham line. After review (`ea2f906`):
  - The line is always walked from the smaller `(x, y)` endpoint, so `a→b` and `b→a` agree. Bresenham's tie-break near a corner differs by direction, but the result no longer does.
  - `World::same_room`: a door tile counts as both its building and outdoors, so a doorway connects the inside to the street. The floor inside and the street outside still don't connect.
  - Room ids use `u8::try_from(i).expect(..)` instead of `as u8`, and `Pos::idx` has a `debug_assert` for off-map positions.
  - Tests cover same-room wall cases in both orders, plus the doorway cases.
- PR #5 (merged, `8b9ad5a`): the apply half of the hearing check: `p_speak`, `holds`, `hearer_confidence`, `contradicts_sighting`, `apply_hop`. Decisions made after review (`18a4afe`), now in `docs/rumor-engine.md` and the README:
  - **Sighting penalty:** −0.10 to the teller on every spoken hop whose hearer already holds a contradicting sighting. It's charged before the drop checks, so a rejected hop still pays, and it's charged again each time the teller repeats the claim. It's charged only in `apply_hop`. `record_sightings` pays only the +0.10 matching bonus, so the same disagreement is never charged twice. A sighting that contradicts a claim heard *earlier* costs nothing.
  - **Contradiction:** the hearer saw someone else hoarding *and* never saw the agent the claim names. A matching sighting cancels it.
  - **`times_told`:** bumped on every spoken hop, accepted or not, so a teller whose neighbours already hold a claim moves on to others.
  - **×2 speak boost:** only while `Busy(Socialize)` at the tavern, not while walking there.
- The user wrote both PRs. Claude reviewed them on GitHub (inline comments, plain comment reviews) and merged #5 after checking the fix commit and running the tests.

## State
- On `main` at `8b9ad5a`.
- `cargo test`: 15 passed, 3 ignored (the `todo!()` test stubs). The build still warns about unused code (e.g. `Sim::plant`) until the engine is wired into `tick`.
- The `hearing-apply` branch was not deleted on merge; delete it with `git push origin --delete hearing-apply`.

## Next
1. Finish the collect phase of the hearing check: `pick_claim`, then `mutate`, then `collect_hops` (it ties the rest together, so last).
2. Wire `hearing_check` into `Sim::tick` (the TODO at the end of `tick()`). Rumors don't move in a real run until this is done.
3. `Action::Hoard`, then `record_sightings`.
4. `drift_earned` and `Proposition::bark`.
5. Serde plus the `--plant`/`--dump` CLI flags (ancestry log and JSON export).
6. Fill in the test stubs: `mutation_rate_and_ancestry`, `broadcast_vs_single_hearer`, `one_sighting_per_watcher`.
7. `belief_feedback`, last, after the tests above pass (per the PRD).

## Notes
- Still open from the last handoff: should the Prophet and Priest start with high loyalty/faith, maybe tied to `prophet_is_fraud`? And stations don't affect behavior yet (tavern keeper at the tavern, prophet and priest favoring the chapel). Both TODOs are on `assign_stations`.
- Workflow: the user writes the Rust and opens a PR, Claude reviews and posts inline comments, the user fixes, and Claude merges with `--merge --delete-branch` once the fix looks good and tests pass.
- No Claude attribution in commits or PRs (user preference).
- `cargo` isn't on PATH in PowerShell; use `~\.cargo\bin\cargo.exe`.
