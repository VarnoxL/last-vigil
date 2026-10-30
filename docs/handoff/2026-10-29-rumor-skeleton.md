# Handoff: 2026-10-29, rumor engine skeleton

## Done
- `sim/src/rumor.rs`: Phase 3 skeleton. Types (`Proposition`, `Origin`, `Claim`, `Known`, `PendingHop`, `RumorEngine`), tuning constants from `docs/rumor-engine.md`, stubbed functions (`todo!()`), and 8 `#[ignore]` test stubs.
- `RumorEngine::plant` was written by the user and reviewed (fixed `teller: Some(agent)` to `None`).
- `agent.rs`: `Station` enum with `base()`; `Agent.station` is always `Townsfolk` for now (TODO: assign from the seed).
- `sim.rs`: `Sim.rumors` and `Sim::plant`. A TODO at the end of `tick()` marks where the hearing check goes. It's not wired in, because the `todo!()`s would panic.
- The README's "Working on this with Claude Code" section was removed.

## State
- On `main`, pushed. Last commit `9fafccb`.
- `cargo test`: 3 passed, 8 ignored. The build warns about unused code until the engine is wired in.
- `cargo` isn't on PATH in this PowerShell; use `~\.cargo\bin\cargo.exe`.

## Next
1. The user writes the plant test: `RumorEngine::new(1, 3)`, plant on agent 0, assert id 0, `root == id`, `teller` is `None`, and `known[0].len() == 1`. It needs `use super::*;` in the tests module.
2. The user writes `influence` (the design doc's Stations and influence section). `p_speak`, `pick_claim` and `hearer_confidence` depend on it.
3. Then `in_range` (room and walls), then the two-phase hearing check.

## Notes
- The user writes the Rust; guide them, don't hand them code (see AGENTS.md).
- Not started: `Action::Hoard` (touches every `match` on `Action`), serde for `--dump`, and the `--plant`/`--dump` CLI flags.
- `Proposition` has the same variants as `sim::Fragment`; merging them is an open option.
- `.claude/settings.json` still enables the Matt Pocock plugin, although the README no longer mentions it.
