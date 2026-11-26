//! Rumor engine (Phase 3). Design: docs/rumor-engine.md; why claims are arena nodes: ADR 0004.
//!
//! Skeleton only: types, tuning numbers, and signatures. Every `todo!()` is a piece to build.
//! Nothing here is called from `Sim::tick` yet, so the existing tests still pass.

// Remove once the hearing check is wired into `Sim::tick`.
#![allow(dead_code, unused_variables)]

use crate::agent::{Agent, Station};
use crate::rng::Rng;
use crate::world::{Pos, World};

// Starting values from the design doc; all of them are meant to be tuned.
/// Ticks between hearing checks.
pub const CHECK_EVERY: u64 = 10;
pub const HEAR_RANGE: u16 = 3;
pub const SIGHT_RANGE: u16 = 6;
/// Confidence kept per hop, before trust and lean.
pub const HOP_DECAY: f32 = 0.85;
/// A hop that lands below this is dropped.
pub const MIN_CONFIDENCE: f32 = 0.1;
/// Influence at or above this tells everyone in range instead of one agent.
pub const BROADCAST_INFLUENCE: f32 = 0.7;
pub const SIGHTING_CONFIDENCE: f32 = 0.95;
pub const EARNED_MIN: f32 = -0.2;
pub const EARNED_MAX: f32 = 0.3;
/// Fraction of earned influence that decays toward 0 per in-game day.
pub const EARNED_DRIFT_PER_DAY: f32 = 0.1;
pub const CORROBORATION_BONUS: f32 = 0.05;
pub const SIGHTING_MATCH_BONUS: f32 = 0.10;
pub const SIGHTING_CONTRADICT_PENALTY: f32 = -0.10;

/// Index into `RumorEngine::claims`.
pub type ClaimId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proposition {
    DoomIsReal(bool),
    ProphetIsFraud(bool),
    DateMiscalculated(bool),
    /// Agent id accused of hoarding.
    Hoarder(usize),
}

impl Proposition {
    /// A canned line for printing and the dump. Claims carry no text until Phase 4.
    pub fn bark(&self) -> String {
        todo!("one line per variant")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Plant,
    Sighting,
    Hop,
}

/// One node in the arena: one agent coming to hold one proposition. Never mutated once pushed.
#[derive(Debug, Clone)]
pub struct Claim {
    pub origin: Origin,
    pub parent: Option<ClaimId>,
    pub root: ClaimId,
    /// None for plants and sightings.
    pub teller: Option<usize>,
    pub hearer: usize,
    pub tick: u64,
    pub confidence: f32,
    pub proposition: Proposition,
}

/// A claim an agent holds, and how often they have passed it on.
#[derive(Debug, Clone)]
pub struct Known {
    pub claim: ClaimId,
    pub times_told: u32,
}

/// A hop decided in the collect phase of a hearing check, applied in the second phase.
#[derive(Debug, Clone)]
pub struct PendingHop {
    pub parent: ClaimId,
    pub teller: usize,
    pub hearer: usize,
    pub confidence: f32,
    /// May differ from the parent's if it mutated.
    pub proposition: Proposition,
}

/// All `Vec`, indexed by id, never `HashMap` (ADR 0003): `Debug` is the determinism fingerprint.
#[derive(Debug)]
pub struct RumorEngine {
    /// Arena; never shrinks.
    pub claims: Vec<Claim>,
    /// `known[agent_id]`.
    pub known: Vec<Vec<Known>>,
    /// Earned influence per agent, in `EARNED_MIN..=EARNED_MAX`.
    pub earned: Vec<f32>,
    rng: Rng,
}

impl RumorEngine {
    pub fn new(seed: u64, agent_count: usize) -> Self {
        Self {
            claims: Vec::new(),
            known: vec![Vec::new(); agent_count],
            earned: vec![0.0; agent_count],
            // Own stream, so rumor draws don't shift the sim's movement draws.
            rng: Rng::new(seed ^ 0x52_554D_4F52), // "RUMOR"
        }
    }

    /// Put a claim straight into `agent`'s head and return its id.
    ///
    /// The new node is a root: no parent, no teller, and `root` is its own id, so every hop that
    /// spreads from it traces back here. Used by tests, `--plant`, and later the god role and omens.
    pub fn plant(&mut self, tick: u64, agent: usize, proposition: Proposition, confidence: f32) -> ClaimId {
        // The id is the index the node is about to land at, so read it before pushing.
        let claim_id = self.claims.len();
        self.claims.push(Claim {
            origin: Origin::Plant,
            parent: None,
            root: claim_id,
            teller: None,
            hearer: agent,
            tick,
            confidence,
            proposition,
        });
        self.known[agent].push(Known { claim: claim_id, times_told: 0 });
        claim_id
    }

    /// `clamp(station_base + derived + earned, 0, 1)`, computed fresh and never stored.
    /// Takes the whole agent list so the id and the agent can't come from different agents.
    /// The engine is always built with the sim's agent count, so `earned` lines up with `agents`.
    pub fn influence(&self, agents: &[Agent], id: usize) -> f32 {
        debug_assert_eq!(self.earned.len(), agents.len(), "RumorEngine built with the wrong agent count");
        let agent = &agents[id];
        let derived = match agent.station {
            Station::Prophet | Station::Priest => 0.2 * (agent.beliefs.loyalty - 0.5),
            _ => 0.1 * (agent.beliefs.fear - 0.5),
        };
        (agent.station.base() + derived + self.earned[id]).clamp(0.0, 1.0)
    }

    /// Run every `CHECK_EVERY` ticks. Collect first from start-of-check state, then apply in order,
    /// so a claim moves at most one hop per check.
    pub fn hearing_check(&mut self, tick: u64, world: &World, agents: &[Agent]) {
        let hops = self.collect_hops(world, agents);
        for hop in hops {
            self.apply_hop(tick, hop, agents);
        }
    }

    /// Phase 1: every agent rolls `p_speak`, picks a claim, picks an audience, and computes
    /// the hearer's confidence and any mutation. Reads state only (plus the rng).
    fn collect_hops(&mut self, world: &World, agents: &[Agent]) -> Vec<PendingHop> {
        todo!()
    }

    /// Phase 2: drop the hop below `MIN_CONFIDENCE` or on a same-root repeat; otherwise push the
    /// node, add it to the hearer's `known`, bump the teller's `times_told`, and credit corroboration.
    fn apply_hop(&mut self, tick: u64, hop: PendingHop, agents: &[Agent]) {
        todo!()
    }

    /// `(0.1 + 0.3·fear + 0.2·loneliness) × (2 if Socializing) × (0.5 + influence)`, capped at 1.
    fn p_speak(&self, agents: &[Agent], id: usize) -> f32 {
        todo!()
    }

    /// Weighted random over `known[teller]` by `confidence / (1 + times_told × (1 − influence))`.
    fn pick_claim(&mut self, teller: usize, influence: f32) -> Option<ClaimId> {
        todo!()
    }

    /// `teller_conf × HOP_DECAY × trust × lean`, capped at 1.
    fn hearer_confidence(&self, claim: ClaimId, teller_influence: f32, hearer: usize, agent: &Agent) -> f32 {
        todo!()
    }

    /// With `p = 0.05 + 0.15 × teller fear`, swap a `Hoarder(id)` to someone in the teller's sight
    /// (or anyone). Yes/no propositions never change.
    fn mutate(&mut self, proposition: Proposition, teller: usize, world: &World, agents: &[Agent]) -> Proposition {
        todo!()
    }

    /// Does `agent` already hold this proposition from this root?
    fn holds(&self, agent: usize, proposition: Proposition, root: ClaimId) -> bool {
        todo!()
    }

    /// Give each agent in sight of a hoarding agent one `Sighting` root at `SIGHTING_CONFIDENCE`,
    /// at most one per proposition, and apply the sighting bonus/penalty to earned influence.
    pub fn record_sightings(&mut self, tick: u64, world: &World, agents: &[Agent]) {
        todo!("needs Action::Hoard first")
    }

    /// Move earned influence toward 0 by `EARNED_DRIFT_PER_DAY`, scaled to the ticks elapsed.
    pub fn drift_earned(&mut self, ticks: u64, ticks_per_day: u64) {
        todo!()
    }
}

/// Same room (same building, or both outdoors), within `range` tiles, with no wall between them.
/// Used for both hearing (`HEAR_RANGE`) and sight (`SIGHT_RANGE`).
pub fn in_range(world: &World, a: Pos, b: Pos, range: u16) -> bool {
    todo!("distance, then room, then walls")
}

/// Apply an accepted claim to the hearer's beliefs. Build last, after the tests below pass.
pub fn belief_feedback(agent: &mut Agent, proposition: Proposition, confidence: f32) {
    todo!()
}

// The design doc's test list. Remove `#[ignore]` from each one as you write it.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plant_makes_a_root_claim() {
        let mut rumors = RumorEngine::new(1, 3);
        let id = rumors.plant(0, 0, Proposition::DoomIsReal(true), 0.9);

        assert_eq!(id, 0);
        let claim = &rumors.claims[id];
        assert_eq!(claim.origin, Origin::Plant);
        assert_eq!(claim.parent, None);
        assert_eq!(claim.root, id);
        assert_eq!(claim.teller, None);
        assert_eq!(claim.hearer, 0);
        assert_eq!(rumors.known[0].len(), 1);
        assert_eq!(rumors.known[0][0].claim, id);
        assert_eq!(rumors.known[0][0].times_told, 0);

        // A second plant gets the next id and its own root, in the other agent's head.
        let second = rumors.plant(5, 2, Proposition::Hoarder(1), 0.5);
        assert_eq!(second, 1);
        assert_eq!(rumors.claims[second].root, second);
        assert_eq!(rumors.claims[second].hearer, 2);
        assert_eq!(rumors.known[2][0].claim, second);
        assert_eq!(rumors.known[0].len(), 1);
    }

    #[test]
    fn influence_formula_and_clamp() {
        let mut rumors = RumorEngine::new(1, 1);
        let mut agents = vec![Agent::new(0, Pos { x: 0, y: 0 }, &mut Rng::new(1))];
        let mut check = |station, fear, loyalty, earned, want: f32| {
            agents[0].station = station;
            agents[0].beliefs.fear = fear;
            agents[0].beliefs.loyalty = loyalty;
            rumors.earned[0] = earned;
            let got = rumors.influence(&agents, 0);
            assert!((got - want).abs() < 1e-6, "{station:?}: got {got}, want {want}");
        };
        check(Station::Townsfolk, 0.5, 0.0, 0.0, 0.3);
        check(Station::Townsfolk, 0.9, 0.0, 0.0, 0.34); // fear coefficient and sign, unclamped
        check(Station::Elder, 0.0, 0.0, 0.1, 0.55); // earned is added, unclamped
        check(Station::Prophet, 0.0, 1.0, 0.3, 1.0); // 1.2 clamps down
        check(Station::Beggar, 0.0, 0.0, -0.2, 0.0); // -0.15 clamps up
        check(Station::Priest, 1.0, 0.0, 0.0, 0.5); // loyalty term, not fear
    }

    #[test]
    #[ignore]
    fn decay_chain_exact_per_hop() {
        todo!()
    }

    #[test]
    #[ignore]
    fn chain_stops_below_cutoff() {
        todo!()
    }

    #[test]
    #[ignore]
    fn same_root_round_trip_adds_nothing() {
        todo!()
    }

    #[test]
    #[ignore]
    fn corroboration_credits_first_teller() {
        todo!()
    }

    #[test]
    #[ignore]
    fn mutation_rate_and_ancestry() {
        todo!()
    }

    #[test]
    #[ignore]
    fn walls_block_hearing_and_sight() {
        todo!()
    }

    #[test]
    #[ignore]
    fn broadcast_vs_single_hearer() {
        todo!()
    }

    #[test]
    #[ignore]
    fn one_sighting_per_watcher() {
        todo!()
    }
}
