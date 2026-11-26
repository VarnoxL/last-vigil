//! Rumor engine (Phase 3). Design: docs/rumor-engine.md; why claims are arena nodes: ADR 0004.
//!
//! Skeleton only: types, tuning numbers, and signatures. Every `todo!()` is a piece to build.
//! Nothing here is called from `Sim::tick` yet, so the existing tests still pass.

// Remove once the hearing check is wired into `Sim::tick`.
#![allow(dead_code, unused_variables)]

use crate::agent::{Action, Agent, Doing, Station};
use crate::rng::Rng;
use crate::world::{Pos, Tile, World};

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
        let root = self.claims[hop.parent].root;
        if hop.confidence < MIN_CONFIDENCE || self.holds(hop.hearer, hop.proposition, root) {
            return;
        }
        // Corroboration: the hearer already holds this proposition, necessarily from another root
        // (same root was rejected above). Credit whoever told them first; nobody if it was a root,
        // and nobody if the first teller is telling again (no corroborating yourself).
        let first = self.known[hop.hearer].iter().find(|k| self.claims[k.claim].proposition == hop.proposition);
        if let Some(t) = first.and_then(|k| self.claims[k.claim].teller).filter(|&t| t != hop.teller) {
            self.earned[t] = (self.earned[t] + CORROBORATION_BONUS).min(EARNED_MAX);
        }
        if self.contradicts_sighting(hop.hearer, hop.proposition) {
            let t = hop.teller;
            self.earned[t] = (self.earned[t] + SIGHTING_CONTRADICT_PENALTY).max(EARNED_MIN);
        }

        let id = self.claims.len();
        self.claims.push(Claim {
            origin: Origin::Hop,
            parent: Some(hop.parent),
            root,
            teller: Some(hop.teller),
            hearer: hop.hearer,
            tick,
            confidence: hop.confidence,
            proposition: hop.proposition,
        });
        self.known[hop.hearer].push(Known { claim: id, times_told: 0 });
        let told = self.known[hop.teller].iter_mut().find(|k| k.claim == hop.parent);
        told.expect("teller holds the claim they told").times_told += 1;
    }

    /// `(0.1 + 0.3·fear + 0.2·loneliness) × (2 if Socializing) × (0.5 + influence)`, capped at 1.
    fn p_speak(&self, agents: &[Agent], id: usize) -> f32 {
        let a = &agents[id];
        // Only once they're at the gathering spot, not while walking there.
        let social = if matches!(a.doing, Doing::Busy(Action::Socialize, _)) { 2.0 } else { 1.0 };
        let base = 0.1 + 0.3 * a.beliefs.fear + 0.2 * a.needs.loneliness;
        (base * social * (0.5 + self.influence(agents, id))).min(1.0)
    }

    /// Weighted random over `known[teller]` by `confidence / (1 + times_told × (1 − influence))`.
    fn pick_claim(&mut self, teller: usize, influence: f32) -> Option<ClaimId> {
        todo!()
    }

    /// `teller_conf × HOP_DECAY × trust × lean`, capped at 1. Takes the proposition as heard,
    /// which differs from `claim`'s if it mutated, since lean depends on the detail.
    fn hearer_confidence(
        &self,
        claim: ClaimId,
        proposition: Proposition,
        teller_influence: f32,
        agents: &[Agent],
        hearer: usize,
    ) -> f32 {
        let faith = agents[hearer].beliefs.faith;
        let mut lean = match proposition {
            Proposition::DoomIsReal(true) => 0.5 + faith,
            Proposition::ProphetIsFraud(true) => 1.5 - faith,
            _ => 1.0,
        };
        if self.contradicts_sighting(hearer, proposition) {
            lean *= 0.3;
        }
        let trust = 0.5 + teller_influence;
        (self.claims[claim].confidence * HOP_DECAY * trust * lean).min(1.0)
    }

    /// The hearer saw someone hoarding, and this claim names someone else.
    /// Only `Hoarder` has a detail to disagree on.
    fn contradicts_sighting(&self, hearer: usize, proposition: Proposition) -> bool {
        let Proposition::Hoarder(named) = proposition else { return false };
        self.known[hearer].iter().any(|k| {
            let c = &self.claims[k.claim];
            c.origin == Origin::Sighting && matches!(c.proposition, Proposition::Hoarder(seen) if seen != named)
        })
    }

    /// With `p = 0.05 + 0.15 × teller fear`, swap a `Hoarder(id)` to someone in the teller's sight
    /// (or anyone). Yes/no propositions never change.
    fn mutate(&mut self, proposition: Proposition, teller: usize, world: &World, agents: &[Agent]) -> Proposition {
        todo!()
    }

    /// Does `agent` already hold this proposition from this root?
    fn holds(&self, agent: usize, proposition: Proposition, root: ClaimId) -> bool {
        self.known[agent].iter().any(|k| {
            let c = &self.claims[k.claim];
            c.proposition == proposition && c.root == root
        })
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
/// Distance is Chebyshev (a square), matching the grid.
pub fn in_range(world: &World, a: Pos, b: Pos, range: u16) -> bool {
    if a.x.abs_diff(b.x).max(a.y.abs_diff(b.y)) > range {
        return false;
    }
    if !world.same_room(a, b) {
        return false;
    }
    // Bresenham can step through different tiles a→b than b→a, so always walk
    // from the smaller endpoint to keep the answer symmetric.
    let (a, b) = if (a.x, a.y) <= (b.x, b.y) { (a, b) } else { (b, a) };
    // Any wall tile on the line blocks. Rooms are plain rectangles, so this
    // only bites outdoors (a line across a building corner).
    let (x1, y1) = (b.x as i32, b.y as i32);
    let (mut x, mut y) = (a.x as i32, a.y as i32);
    let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs());
    let (sx, sy) = ((x1 - x).signum(), (y1 - y).signum());
    let mut err = dx + dy;
    loop {
        if world.tile(Pos { x: x as u16, y: y as u16 }) == Tile::Wall {
            return false;
        }
        if x == x1 && y == y1 {
            return true;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
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

    /// `n` townsfolk at fear 0.5, so each has influence exactly 0.3 (trust 0.8).
    fn town(n: usize) -> (RumorEngine, Vec<Agent>) {
        let mut rng = Rng::new(7);
        let agents = (0..n)
            .map(|_| {
                let mut a = Agent::new(0, Pos { x: 0, y: 0 }, &mut rng);
                a.beliefs.fear = 0.5;
                a
            })
            .collect();
        (RumorEngine::new(1, n), agents)
    }

    /// One unmutated hop of the claim `teller` holds last, computed the way the collect phase will.
    fn tell(r: &mut RumorEngine, agents: &[Agent], teller: usize, hearer: usize) {
        let parent = r.known[teller].last().expect("teller knows something").claim;
        let proposition = r.claims[parent].proposition;
        let confidence = r.hearer_confidence(parent, proposition, r.influence(agents, teller), agents, hearer);
        r.apply_hop(0, PendingHop { parent, teller, hearer, confidence, proposition }, agents);
    }

    const PER_HOP: f32 = HOP_DECAY * 0.8; // trust 0.8, lean 1

    #[test]
    fn decay_chain_exact_per_hop() {
        // A yes/no proposition with lean 1.0: never mutates, so the chain is pure decay.
        let (mut r, agents) = town(4);
        let root = r.plant(0, 0, Proposition::DateMiscalculated(true), 1.0);
        for i in 0..3 {
            tell(&mut r, &agents, i, i + 1);
        }
        let mut want = 1.0;
        for (id, c) in r.claims.iter().enumerate().skip(1) {
            want *= PER_HOP;
            assert!((c.confidence - want).abs() < 1e-6, "hop {id}: got {}, want {want}", c.confidence);
            assert_eq!((c.origin, c.parent, c.root), (Origin::Hop, Some(id - 1), root));
            assert_eq!((c.teller, c.hearer), (Some(id - 1), id));
            assert_eq!(r.known[id - 1].last().unwrap().times_told, 1);
        }
    }

    #[test]
    fn lean_and_cap() {
        let (mut r, mut agents) = town(2);
        agents[1].beliefs.faith = 0.2;
        let doom = r.plant(0, 0, Proposition::DoomIsReal(true), 1.0);
        let got = r.hearer_confidence(doom, Proposition::DoomIsReal(true), 0.3, &agents, 1);
        assert!((got - PER_HOP * 0.7).abs() < 1e-6, "doom lean 0.5 + faith: {got}");
        let fraud = r.plant(0, 0, Proposition::ProphetIsFraud(true), 1.0);
        let got = r.hearer_confidence(fraud, Proposition::ProphetIsFraud(true), 0.3, &agents, 1);
        assert!((got - PER_HOP * 1.3).abs() < 1e-6, "fraud lean 1.5 - faith: {got}");
        // Trust 1.5 and lean 1.3 push it past 1.
        assert_eq!(r.hearer_confidence(fraud, Proposition::ProphetIsFraud(true), 1.0, &agents, 1), 1.0);
    }

    #[test]
    fn contradicting_sighting_cuts_lean_and_costs_teller() {
        let (mut r, agents) = town(2);
        // No sightings exist until Action::Hoard, so fake one: agent 1 saw agent 5 hoarding.
        r.claims.push(Claim {
            origin: Origin::Sighting,
            parent: None,
            root: 0,
            teller: None,
            hearer: 1,
            tick: 0,
            confidence: SIGHTING_CONFIDENCE,
            proposition: Proposition::Hoarder(5),
        });
        r.known[1].push(Known { claim: 0, times_told: 0 });
        r.plant(0, 0, Proposition::Hoarder(9), 1.0);
        tell(&mut r, &agents, 0, 1);
        assert!((r.claims[2].confidence - PER_HOP * 0.3).abs() < 1e-6);
        assert!((r.earned[0] - SIGHTING_CONTRADICT_PENALTY).abs() < 1e-6);
    }

    #[test]
    fn chain_stops_below_cutoff() {
        // 0.68^n first drops below 0.1 at n = 6, so hops 1..=5 land and the 6th is dropped.
        let (mut r, agents) = town(7);
        r.plant(0, 0, Proposition::DateMiscalculated(true), 1.0);
        for i in 0..6 {
            tell(&mut r, &agents, i, i + 1);
        }
        assert_eq!(r.claims.len(), 6);
        assert!(r.claims.iter().all(|c| c.confidence >= MIN_CONFIDENCE));
        assert!(r.known[6].is_empty());
    }

    #[test]
    fn same_root_round_trip_adds_nothing() {
        let (mut r, agents) = town(2);
        r.plant(0, 0, Proposition::DateMiscalculated(true), 1.0);
        tell(&mut r, &agents, 0, 1);
        tell(&mut r, &agents, 1, 0);
        assert_eq!(r.claims.len(), 2);
        assert_eq!(r.known[0].len(), 1);
        assert_eq!(r.known[1][0].times_told, 0, "a rejected hop doesn't count as told");
    }

    #[test]
    fn corroboration_credits_first_teller() {
        let p = Proposition::DateMiscalculated(true);
        let (mut r, agents) = town(4);
        r.plant(0, 0, p, 1.0);
        r.plant(0, 2, p, 1.0);
        r.earned[0] = 0.28;
        tell(&mut r, &agents, 0, 1); // agent 1's first: from 0
        tell(&mut r, &agents, 2, 1); // same proposition, other root: kept, and 0 gets credit
        assert_eq!(r.known[1].len(), 2);
        assert_eq!(r.earned[0], EARNED_MAX, "0.28 + 0.05 clamps to 0.3");
        assert_eq!(r.earned[2], 0.0);

        // Agent 3 first got it as a plant, so a second root credits nobody.
        r.plant(0, 3, p, 1.0);
        tell(&mut r, &agents, 2, 3);
        assert_eq!(r.known[3].len(), 2);
        assert_eq!(r.earned[2], 0.0);
        assert_eq!(r.earned[0], EARNED_MAX);
    }

    #[test]
    fn no_corroborating_yourself() {
        let p = Proposition::DateMiscalculated(true);
        let (mut r, agents) = town(2);
        r.plant(0, 0, p, 1.0);
        tell(&mut r, &agents, 0, 1);
        r.plant(0, 0, p, 1.0); // a second root, also in agent 0's head
        tell(&mut r, &agents, 0, 1);
        assert_eq!(r.known[1].len(), 2);
        assert_eq!(r.earned[0], 0.0);
    }

    #[test]
    #[ignore]
    fn mutation_rate_and_ancestry() {
        todo!()
    }

    #[test]
    fn walls_block_hearing_and_sight() {
        // First home: rect x 2..=6, y 18..=21, top wall at y = 18.
        let world = World::town();
        let p = |x, y| Pos { x, y };

        // Home floor vs grass just outside, 2 tiles apart through the top wall.
        for range in [HEAR_RANGE, SIGHT_RANGE] {
            assert!(!in_range(&world, p(4, 19), p(4, 17), range));
        }
        // Control: two outdoor tiles 2 apart with nothing between.
        assert!(in_range(&world, p(4, 17), p(6, 17), HEAR_RANGE));
        // Both outdoors, so only the wall check decides. Each pair both ways round.
        // The line cuts the home's top-left corner (2, 18): blocked.
        assert!(!in_range(&world, p(1, 19), p(3, 17), SIGHT_RANGE));
        assert!(!in_range(&world, p(3, 17), p(1, 19), SIGHT_RANGE));
        // Grazes the corner; Bresenham's tie-break differs by direction, the answer must not.
        assert!(in_range(&world, p(1, 18), p(3, 17), SIGHT_RANGE));
        assert!(in_range(&world, p(3, 17), p(1, 18), SIGHT_RANGE));

        // Home 0's door (4, 21) is in both rooms; floor and street are not.
        assert!(in_range(&world, p(4, 21), p(4, 22), HEAR_RANGE));
        assert!(in_range(&world, p(4, 22), p(4, 21), HEAR_RANGE));
        assert!(in_range(&world, p(4, 21), p(4, 20), HEAR_RANGE));
        assert!(!in_range(&world, p(4, 20), p(4, 22), HEAR_RANGE));
        // Clear line, but 4 tiles apart.
        assert!(!in_range(&world, p(4, 17), p(8, 17), HEAR_RANGE));
        // TODO: extend to the doc's "no hop and no sighting" once the hearing check exists.
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
