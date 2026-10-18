# Rumor engine design (Phase 3)

Settled in a design session on 2026-10-18. Terms are defined in `GLOSSARY.md`; why claims are arena nodes is in ADR 0004. All numbers are starting values to tune.

## Data

```rust
// rumor.rs
pub type ClaimId = usize; // index into `claims`

pub enum Proposition { DoomIsReal(bool), ProphetIsFraud(bool), DateMiscalculated(bool), Hoarder(usize) }
pub enum Origin { Plant, Sighting, Hop }

pub struct Claim {
    pub origin: Origin,
    pub parent: Option<ClaimId>,
    pub root: ClaimId,
    pub teller: Option<usize>, // None for plants and sightings
    pub hearer: usize,
    pub tick: u64,
    pub confidence: f32,
    pub proposition: Proposition,
}

pub struct Known { pub claim: ClaimId, pub times_told: u32 }

pub struct RumorEngine {
    pub claims: Vec<Claim>,     // arena, never shrinks
    pub known: Vec<Vec<Known>>, // known[agent_id]
    pub earned: Vec<f32>,       // earned influence per agent
    rng: Rng,                   // forked from the seed: Rng::new(seed ^ CONST)
}
```

- `Agent` gains only `station: Station`. `Sim` gains `rumors: RumorEngine`.
- `Proposition::bark()` returns a canned line for printing and the dump. Claims store no text until Phase 4.
- Everything is `Vec`, indexed by id, and never a `HashMap` (ADR 0003).

## Stations and influence

| Station | Count | Base |
|---|---|---|
| Prophet | 1 | 0.8 |
| Priest | 1 | 0.6 |
| Tavern keeper | 1 | 0.5 |
| Elder | 2 | 0.5 |
| Townsfolk | 42 | 0.3 |
| Beggar | 3 | 0.1 |

Stations are assigned from the seed. The tavern keeper works at the tavern, and the prophet and priest favor the chapel. The hoarder can be any station.

```
influence = clamp(station_base + derived + earned, 0, 1)    // computed fresh, never stored
derived   = 0.2 × (loyalty − 0.5)   prophet and priest
            0.1 × (fear − 0.5)      everyone else
earned    ∈ [−0.2, +0.3], drifts toward 0 at 10% per in-game day
            +0.05  corroboration: hearer gets the same proposition from a different root;
                   credited to whoever told the hearer first (none if that was a sighting)
            +0.10  a hearer sights something matching a claim you told them
            −0.10  your claim reaches a hearer whose sighting contradicts it
```

## Hearing check (every 10 ticks)

Two phases. First, collect `Vec<PendingHop>` reading only the state at the start of the check. Then apply them all in order. A claim moves at most one hop per check.

- **Range:** hearing is ≤ 3 tiles and sight is ≤ 6 tiles. Both require the same room (same building, or both outdoors), and walls block them.
- **Speak:** `p_speak = (0.1 + 0.3·fear + 0.2·loneliness) × (2 if Socializing else 1) × (0.5 + influence)`, capped at 1.
- **Pick a claim:** weighted random by `confidence / (1 + times_told × (1 − influence))`.
- **Audience:** influence ≥ 0.7 broadcasts to everyone in hearing range. Otherwise the teller picks one random agent in range.
- **Hearer confidence:** `teller_conf × 0.85 × trust × lean`, capped at 1.
  - `trust = 0.5 + teller influence`
  - `lean`: `DoomIsReal(true)` → `0.5 + faith`; `ProphetIsFraud(true)` → `1.5 − faith`; otherwise `1.0`
  - Multiply `lean` by 0.3 if the hearer holds a sighting of the same kind with a different detail.
- **Reject the hop** if the result is below 0.1, or if the hearer already holds the same proposition with the same root.
- **Keep duplicates** that come from a different root; corroboration fires on them.
- **Mutation:** `p = 0.05 + 0.15 × teller fear`. It swaps only `Hoarder(id)`: to a random agent in the teller's sight range, otherwise any agent. Yes/no claims never flip. The mutated node keeps `parent` and `root`.

## Hoarding and sightings

- `Action::Hoard` means walking to the market and carrying supplies home. Its utility rises with urgency.
- **Sneaky:** the hoarder starts only if the number of agents in sight is ≤ `floor(urgency × 3)`. They can still be seen on the walk home.
- **Sighting:** an agent in sight range of a hoarding agent gets a `Sighting` claim `Hoarder(id)` at high confidence (about 0.95). Each agent gets at most one sighting per proposition.

## Belief feedback (build last)

On accepting a claim with confidence `c`:

```
DoomIsReal(true)        faith   += 0.1  × c × (1 − faith)
ProphetIsFraud(true)    loyalty −= 0.1  × c × loyalty;  faith −= 0.05 × c × faith
DateMiscalculated(true) faith   −= 0.05 × c × faith
Hoarder(_)              fear    += 0.05 × c × (1 − fear)
```

## Entry points

- `Sim::plant(agent, proposition, confidence)` adds a root with `origin: Plant`. It is used by tests, the CLI, and later by the god role and omens.
- `--plant hoarder:7` plants a claim at tick 0.
- `--dump <file>` writes `{ seed, tick, claims: [...] }` at the end of a headless run, one flat node per claim (with its `id` and `bark`), using serde/serde_json.

## Tests

1. Decay chain: confidence is exact per hop, with mutation off.
2. Cutoff: a chain stops at the first hop below 0.1.
3. Same-root skip: A→B→A adds no node.
4. Corroboration: two independent plants reach one hearer, and the first teller gets +0.05.
5. Mutation: about 20% at fear 1, never on bools, and `parent`/`root` are kept.
6. Walls: a pair 2 tiles apart through a wall gets no hop and no sighting.
7. Broadcast: influence 0.8 reaches all agents in range, and influence 0.3 reaches exactly one.
8. Sighting: a watcher gets exactly one sighting, however long they watch.

The existing determinism test must still pass. The manual exit check is to plant a claim, run 10 sim minutes, `--dump`, and confirm the tree has more than one hop.
