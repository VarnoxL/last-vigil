# Last Vigil — PRD

> Branch: main · Worktree: C:\Users\danie\Desktop\last-vigil · Last updated: 2026-10-18

## Problem

Most "AI agent village" projects are cozy, scripted-feeling demos where every agent calls a big cloud model and nothing is at stake. There is no good sandbox for watching how *information* (and misinformation) moves through a society under pressure, and none that runs on a single consumer GPU.

This project is a game-like simulation of a haunted medieval town built around a doomsday cult. A countdown runs toward a prophesied end. Whether the doom is real is a hidden truth, randomized per run. Agents know only what they have seen or been told, so rumors spread, mutate, and get exploited while the town panics, denies, or finds faith. It is also a learning vehicle for its author: Rust, TypeScript, Python, local model serving, and Three.js in one real system.

## Goals

- A tiered agent society (50+ agents) running on one RTX 3060 Ti (8 GB) with local models only.
- A rumor engine where every rumor is traceable (who said what to whom, with what confidence) *and* organically distorted in wording.
- A hidden truth per run, with a doomsday countdown that visibly changes agent behavior as it nears zero.
- A pixel-art, game-like Three.js client (not a dashboard).
- Four selectable roles on the same sim: observer, invisible god, cult member, investigator.
- Each language does the job it is best at: Rust (sim), Python (model-side), TypeScript (client).

## Non-goals

- Cloud/API models (Claude, GPT) in the core loop. Local only.
- Multiplayer or online hosting.
- Custom-drawn art, AI-generated art, or procedural sprites (free CC0 packs first).
- 3D or isometric rendering. Top-down 2D pixel look only.
- Hostile ghost combat/threat system. Ghosts are informational, not violent.
- Cross-run persistence (ghosts as past-run dead). Possible later; not planned.
- A scripted fixed storyline. Every run generates its own hidden truth.

## Success criteria

- 50+ agents simulated at 10 ticks/s of sim time with Tier 0 cost under 2 ms/tick on the author's machine.
- A rumor planted in one run can be traced end to end in the UI: origin, every hop, per-hop confidence, per-hop wording.
- Two runs with different seeds produce different hidden truths and visibly different town behavior near zero.
- Tier 1 + Tier 2 model calls keep the sim from stalling: no agent decision blocks the tick loop; p95 decision latency is shown in a debug overlay.
- The same seed with models stubbed reproduces identical sim state (determinism of the Rust core).
- A live run's replay log replays to an identical final state with no models loaded.
- A run can be started with any of the three run-length presets (Quick, Standard, Long) from the start screen.
- All four roles are playable in one build, switchable from a start screen.
- README contains real rendered images of the world, produced by the project's own tooling.

## Technical approach

**Architecture.** Three processes talking over local sockets:

| Piece | Language | Why |
|---|---|---|
| `sim/` world state, tick loop, rumor propagation, spatial hearing, tier scheduling, promotion | **Rust** | Deterministic, fast for 50+ agents, and the learning goal. Runs as a native server so it is debuggable and not tied to the browser. |
| `brain/` model gateway, prompt templates, eval harness, embedding memory retrieval, rumor-spread analysis notebooks (and later a LoRA fine-tune, Phase 11) | **Python** | Best ML tooling. Only needs to speak HTTP to Ollama/llama.cpp and a local socket to the sim. |
| `client/` Three.js renderer, UI, role input layers, websocket client | **TypeScript** | Browser-native, shared types for the wire protocol. |

- **Sim ↔ client:** websocket, JSON snapshots/deltas. **Sim ↔ brain:** local HTTP (sim sends decision requests, brain returns structured results). Rust never calls the LLM directly; this keeps all prompting in Python.
- **Tiers.** T0 crowd (30–50): no LLM, utility-AI state machines, beliefs as numbers, canned barks. T1 townsfolk (10–15): 1–3B model, event-driven (hear rumor, meet someone). T2 key figures (3–5): 7–8B model with memory retrieval and reflection. *Static tiers first; dynamic promotion/demotion by narrative involvement comes after the basics work.*
- **Rumor model (hybrid).** A rumor is a structured claim `(id, subject, predicate, source, confidence, hops, truth_link)` that Rust tracks and mutates (confidence decay, detail swaps). On each hop, the brain rewrites the wording, so ancestry stays queryable while text distorts organically. T0 agents repeat the nearest canned form.
- **Hidden truth.** Generated at run start from a seed: e.g. prophet is a fraud, someone hoards supplies, the date was a miscalculation, the doom is real. One field is `doom_is_real`, randomized. Agents never see this; ghosts hold true fragments.
- **Ghosts.** Know past events, cannot speak plainly. They emit omens, dreams, and speak through mediums; each message enters the rumor engine as a new claim with low clarity.
- **Doomsday countdown.** A global clock agents are aware of. Fear, loyalty, and belief parameters shift as it approaches; behaviors include panic, hoarding, flight, penance, denial, mutiny. At zero, outcome follows `doom_is_real`.
- **Roles.** Observer (no input), God (whisper a claim to an agent or area), Cult Member (walk, talk, influence), Investigator (question agents, collect unreliable testimony, accuse). All roles are input layers over the same sim protocol; the sim treats human actions as events.
- **Run length.** Chosen per run on the start screen from three presets: Quick (~10–15 min), Standard (~30–45 min), Long (hours). A preset is just two numbers, countdown days and real minutes per in-game day. The brain's throughput is fixed (one GPU, roughly 1–3 s per call), so a longer run means more decisions per agent per day, not a faster GPU. Long runs exist for gathering data.
- **Replay log.** Live model replies are nondeterministic (latency and wording vary), so the sim appends every brain reply, along with the tick it was applied on, to a per-run log. In replay mode the sim reads replies from the log instead of calling the brain, which reproduces the run exactly without a GPU. This is used for debugging, re-watching good runs, README screenshots, and Phase 11 training data. It requires two rules in Rust: brain replies are applied only at tick boundaries, in a fixed order, and sim state never iterates a `HashMap` (use `BTreeMap`/`Vec`). See ADR 0003.
- **Models.** Ollama or llama.cpp server. 8 GB VRAM fits a 7–8B Q4 (~5 GB), but the browser and Windows also take about 1 GB. The tier-to-model mapping lives in brain config, and the Phase 1 smoke test picks it with this rule: if the 3B and 8B models both sit fully on the GPU with at least 1 GB to spare, keep two models; otherwise all tiers share one (qwen3:8b, or qwen3:4b if speed is poor) and tiers differ only by call frequency, context, memory, and reflection.
- **Art.** CC0 tilesets and sprite packs. Orthographic Three.js camera with nearest-neighbor textures; fog, lighting, and ghost transparency done in shaders.
- **Why not all one language:** the author explicitly wants to learn all three, and each has a genuine best-fit job above.

## Risks / open questions

- **VRAM co-residency** of a 1–3B and a 7–8B model on 8 GB is untested. Settled by the smoke-test rule in Technical approach. Embedding memory (T2) needs its own small model; if it doesn't fit, embeddings run on the CPU.
- **Small-model quality.** 1–3B models may produce incoherent rumor rewrites. Mitigation: constrained/structured outputs and a Python eval harness; accept some silliness as a feature.
- **Throughput.** Measured at ~4.7 s per decision (≈750 decisions per real hour, serialized), slower than the 1–3 s originally estimated. Bursty rumor events could back up the queue; T1 calls should use shorter outputs, and Quick runs will give each agent only a handful of decisions. Needs a priority queue and a cheap fallback behavior; exact policy is unresolved.
- **Promotion/demotion rules** (what counts as "narrative involvement") are not designed yet; deferred to Phase 8.
- **Cult-member and investigator interaction UI** (free-text vs. choice menus) is not decided; to be settled when Phase 9 starts.
- **Wire-protocol types** shared between Rust and TypeScript need one source of truth (e.g. generating TS types from Rust structs). Not decided; settle at the start of Phase 5.
- **Toolchain:** Rust 1.99.1 and Ollama 0.40.5 are installed (2026-10-17), but Rust on Windows also needs the MSVC Build Tools linker, which was still installing at the time of writing.
- **Asset licensing:** each chosen CC0 pack must be recorded in `ASSETS.md` before use.

## Phases

### Phase 1 — Toolchain and skeleton
**Depends on:** none
**Done when:** `cargo run` in `sim/` prints a tick counter, `python -m brain` answers a health check, and `npm run dev` in `client/` serves a blank canvas.
- [x] Install Rust toolchain (rustup), MSVC Build Tools, and Ollama; `cargo run` links
- [x] Agent smoke test (`brain/agent_smoke_test.py`): a T1 and a T2 persona each return a schema-valid rumor decision; record latency, tok/s, and VRAM, then apply the model rule (two models vs. one)
  - Result 2026-10-17: PASS. Co-residency failed (Ollama evicts the 3B when the 8B loads; 8B alone uses 6.4 / 8 GB). **Model rule → one shared `qwen3:8b` for all tiers.** Warm 8B: ~4.7 s per decision, ~60 tok/s, 100% on GPU. Cold load ~30–70 s, so the brain must preload and keep the model alive.
- [x] Create `sim/` Cargo project, `brain/` Python package, `client/` Vite + TypeScript + Three.js project
- [x] Add `ASSETS.md` and a root `README.md` stub

### Phase 2 — Headless sim core (Tier 0 only)
**Depends on:** Phase 1
**Done when:** a seeded run of 50 agents on a tile map ticks for 10 simulated minutes with deterministic, identical final state across two runs.
- [x] World map, tiles, buildings, and a pathfinding grid
- [x] Agent state: needs, position, beliefs as numbers, utility-AI action selection
- [x] Seeded RNG and a determinism test
- [x] Doomsday clock driven by a run-length preset (days, real minutes per day) and the hidden-truth generator (`doom_is_real` and truth fragments)
  - Result 2026-10-18: 50 agents, 6000 ticks (10 sim minutes), seed 42 → identical final state across two runs (`cargo test`). T0 cost ~0.0012 ms/tick in release, far under the 2 ms budget. Presets: Quick 3 days × 4 min, Standard 5 × 8, Long 10 × 20.

### Phase 3 — Rumor engine
**Depends on:** Phase 2
**Done when:** a claim planted in a headless run spreads by proximity, mutates in confidence per hop, and can be dumped as an ancestry tree.
- [ ] Claim data structure and spatial hearing rules
- [ ] Per-hop confidence decay and detail-swap mutation
- [ ] Ancestry log and JSON export
- [ ] Unit tests for propagation and mutation

### Phase 4 — Brain service and Tier 1/2 agents
**Depends on:** Phase 3
**Done when:** with models live, a T1 agent hearing a rumor produces a structured believe/repeat decision plus rewritten wording, and a T2 agent produces a reflection that changes its later behavior, without stalling the tick loop.
- [ ] Python gateway to Ollama with structured (schema-constrained) outputs
- [ ] Sim ↔ brain protocol and async request queue with priorities and timeouts
- [ ] Replay log: record every applied brain reply with its tick; replay mode reads from the log; test that a live run replays to an identical final state
- [ ] T1 event-driven decisions and T2 memory retrieval + reflection
- [ ] Eval harness comparing prompts and models on rumor-rewrite quality
- [ ] Re-check the model rule under real load (sim + browser running)

### Phase 5 — Three.js world (observer mode)
**Depends on:** Phase 3 (Phase 4 can land in parallel)
**Done when:** the browser shows the pixel-art town with agents walking, speech bubbles, a doomsday countdown HUD, and a clickable rumor-ancestry panel.
- [ ] Websocket client and snapshot/delta protocol
- [ ] Orthographic pixel renderer, tilemap, sprite animation
- [ ] Day/night, fog, and ghost shaders
- [ ] Speech bubbles and rumor trace overlay
- [ ] Start screen with seed entry, run-length preset, and "open replay"

### Phase 6 — Ghosts and the haunted layer
**Depends on:** Phases 4 and 5
**Done when:** ghosts emit omens and medium-channeled messages that appear in the world and enter the rumor engine as low-clarity claims.
- [ ] Ghost agent type with past-knowledge and cryptic-output rules
- [ ] Omen and dream events and medium agents
- [ ] Visual treatment (translucency, flicker, whisper text)

### Phase 7 — God role
**Depends on:** Phase 5
**Done when:** the player can select an agent or area, whisper a claim, and watch it enter the rumor engine and spread.
- [ ] Role selector on the start screen
- [ ] Whisper UI and a human-action event type in the sim protocol
- [ ] Trace highlighting for god-originated rumors

### Phase 8 — Tier dynamics (promotion/demotion)
**Depends on:** Phase 4
**Done when:** an agent central to a rumor chain is promoted a tier mid-run and demoted when it quiets down, visible in a debug overlay.
- [ ] Define and tune an involvement score
- [ ] Promotion/demotion with state hand-off between tiers
- [ ] Throughput metrics in the debug overlay

### Phase 9 — Cult member role
**Depends on:** Phases 5 and 8
**Done when:** the player controls a cult member who walks, converses, and influences beliefs, and agents react to the player's words.
- [ ] Player-controlled agent and movement
- [ ] Conversation input (decide free-text vs. menu) and agent responses
- [ ] Influence and suspicion mechanics

### Phase 10 — Investigator role and endings
**Depends on:** Phases 6 and 9
**Done when:** the player can interview agents, collect testimony, make an accusation, and see the run end at doomsday with an outcome that matches the hidden truth.
- [ ] Testimony log and evidence board
- [ ] Accusation and resolution against the hidden truth
- [ ] Doomsday ending sequences (real, hoax, other)
- [ ] Final README pass with captured screenshots from the real client

### Phase 11 — In-character LoRA (optional stretch)
**Depends on:** Phase 4 (replay logs as data); best after Phase 10
**Done when:** a QLoRA-tuned 3B model, served by Ollama, beats the base model on the eval harness's rumor-rewrite and in-character scores.
- [ ] Build a training set from the best replay-log lines
- [ ] QLoRA fine-tune of the 3B model on the 3060 Ti and import it into Ollama
- [ ] Before/after comparison in the eval harness
