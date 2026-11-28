use crate::agent::{Action, Agent, Doing, SPECIAL_STATIONS};
use crate::rng::Rng;
use crate::rumor::{in_range, ClaimId, Proposition, RumorEngine, CHECK_EVERY, SIGHT_RANGE};
use crate::world::{BuildingKind, Pos, World};

pub const TICKS_PER_SEC: u64 = 10;
/// Ticks per tile walked (0.3 s).
const STEP_TICKS: u64 = 3;

/// Run length: countdown days and real minutes per in-game day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Quick,
    Standard,
    Long,
}

impl Preset {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "quick" => Some(Self::Quick),
            "standard" => Some(Self::Standard),
            "long" => Some(Self::Long),
            _ => None,
        }
    }

    /// (countdown days, real minutes per day)
    pub fn shape(self) -> (u64, u64) {
        match self {
            Self::Quick => (3, 4),     // 12 min
            Self::Standard => (5, 8),  // 40 min
            Self::Long => (10, 20),    // 3 h 20 min
        }
    }

    pub fn ticks_per_day(self) -> u64 {
        self.shape().1 * 60 * TICKS_PER_SEC
    }

    pub fn total_ticks(self) -> u64 {
        self.shape().0 * self.ticks_per_day()
    }
}

/// What is actually going on in this run. No agent ever reads this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Truth {
    pub doom_is_real: bool,
    pub prophet_is_fraud: bool,
    pub date_miscalculated: bool,
    /// Agent id secretly hoarding supplies.
    pub hoarder: Option<usize>,
}

/// One piece of the hidden truth, as a ghost would hold it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fragment {
    DoomIsReal(bool),
    ProphetIsFraud(bool),
    DateMiscalculated(bool),
    Hoarder(usize),
}

impl Truth {
    pub fn generate(rng: &mut Rng, agents: usize) -> Self {
        let doom_is_real = rng.chance(0.5);
        // A false doom needs an explanation: fraud, a bad calculation, or both.
        let (prophet_is_fraud, date_miscalculated) = if doom_is_real {
            (false, false)
        } else {
            let fraud = rng.chance(0.6);
            (fraud, !fraud || rng.chance(0.3))
        };
        let hoarder = rng.chance(0.5).then(|| rng.below(agents));
        Self { doom_is_real, prophet_is_fraud, date_miscalculated, hoarder }
    }

    pub fn fragments(&self) -> Vec<Fragment> {
        let mut f = vec![
            Fragment::DoomIsReal(self.doom_is_real),
            Fragment::ProphetIsFraud(self.prophet_is_fraud),
            Fragment::DateMiscalculated(self.date_miscalculated),
        ];
        f.extend(self.hoarder.map(Fragment::Hoarder));
        f
    }
}

/// The whole sim state. `Debug` output doubles as the determinism fingerprint,
/// so keep every field ordered (`Vec`/`BTreeMap`, never `HashMap`; see ADR 0003).
#[derive(Debug)]
pub struct Sim {
    pub tick: u64,
    pub preset: Preset,
    pub truth: Truth,
    pub world: World,
    pub agents: Vec<Agent>,
    pub rumors: RumorEngine,
    rng: Rng,
}

impl Sim {
    pub fn new(seed: u64, preset: Preset, agent_count: usize) -> Self {
        let mut rng = Rng::new(seed);
        let world = World::town();
        let truth = Truth::generate(&mut rng, agent_count);
        let homes: Vec<usize> =
            (0..world.buildings.len()).filter(|&i| world.buildings[i].kind == BuildingKind::Home).collect();
        let mut agents: Vec<Agent> = (0..agent_count)
            .map(|id| {
                let home = homes[id % homes.len()];
                Agent::new(home, world.buildings[home].door, &mut rng)
            })
            .collect();
        assign_stations(seed, &mut agents);
        let rumors = RumorEngine::new(seed, agent_count);
        Self { tick: 0, preset, truth, world, agents, rumors, rng }
    }

    pub fn plant(&mut self, agent: usize, proposition: Proposition, confidence: f32) -> ClaimId {
        self.rumors.plant(self.tick, agent, proposition, confidence)
    }

    /// 0 at run start, 1 at countdown zero.
    pub fn urgency(&self) -> f32 {
        (self.tick as f32 / self.preset.total_ticks() as f32).min(1.0)
    }

    pub fn countdown_left(&self) -> u64 {
        self.preset.total_ticks().saturating_sub(self.tick)
    }

    pub fn tick(&mut self) {
        self.tick += 1;
        let tick = self.tick;
        let day = self.preset.ticks_per_day() as f32;
        let urgency = self.urgency();
        // Counted before the loop, from start-of-tick positions: the loop holds `agents` mutably.
        // Only an idle hoarder is about to choose, so only then is it worth counting.
        let hoarder = self.truth.hoarder.filter(|&h| matches!(self.agents[h].doing, Doing::Idle)).filter(|&h| {
            let here = self.agents[h].pos;
            let watchers = (0..self.agents.len())
                .filter(|&i| i != h && in_range(&self.world, here, self.agents[i].pos, SIGHT_RANGE))
                .count();
            hoard_gate(watchers, urgency)
        });
        let Sim { agents, world, rng, .. } = self;

        // Fixed agent order: every rng draw happens in the same sequence each run.
        for (id, a) in agents.iter_mut().enumerate() {
            // Needs fill over a fraction of a day, so behavior per day is the same for every preset.
            let n = &mut a.needs;
            n.hunger = (n.hunger + 3.0 / day).min(1.0);
            n.fatigue = (n.fatigue + 1.5 / day).min(1.0);
            n.loneliness = (n.loneliness + 2.0 / day).min(1.0);

            // Fear drifts toward what the countdown means to this agent: believers dread it, doubters a little.
            let b = &mut a.beliefs;
            b.fear += (urgency * (0.2 + 0.8 * b.faith) - b.fear) * (8.0 / day);

            a.doing = match std::mem::replace(&mut a.doing, Doing::Idle) {
                Doing::Idle => {
                    let action = a.choose(rng, hoarder == Some(id), urgency);
                    let goal = goal(world, a, action, rng);
                    match world.path(a.pos, goal) {
                        Some(path) => Doing::Walking(action, path),
                        None => Doing::Idle,
                    }
                }
                Doing::Walking(action, mut path) => {
                    if tick % STEP_TICKS == 0 {
                        if let Some(p) = path.pop() {
                            a.pos = p;
                        }
                    }
                    if path.is_empty() { Doing::Busy(action, duration(action, day)) } else { Doing::Walking(action, path) }
                }
                // Before the general `Busy(_, 0)` arm, which would otherwise catch it.
                Doing::Busy(Action::Hoard, 0) => match world.path(a.pos, world.buildings[a.home].door) {
                    Some(path) => Doing::Walking(Action::CarryHome, path),
                    None => Doing::Idle,
                },
                Doing::Busy(action, 0) => {
                    a.finish(action);
                    Doing::Idle
                }
                Doing::Busy(action, left) => Doing::Busy(action, left - 1),
            };
        }

        if tick % CHECK_EVERY == 0 {
            self.rumors.hearing_check(tick, &self.world, &self.agents);
            // After the hops, so a claim told this check and sighted now pays the match bonus.
            self.rumors.record_sightings(tick, &self.world, &self.agents);
            // TODO(phase 3): `drift_earned(...)` here once built.
        }
    }

    /// Text frame of the town: `#` wall, `+` door, `=` road, `.` grass, and each agent
    /// as the letter of what it's doing (Eat, Sleep, sOcialize, Pray, Wander, Hoard, Carry home, Idle).
    /// A stand-in view until the Three.js client lands in Phase 5.
    pub fn render(&self) -> String {
        use crate::world::{Tile, H, W};
        let mut grid: Vec<char> = self
            .world
            .tiles
            .iter()
            .map(|t| match t {
                Tile::Grass => '.',
                Tile::Road => '=',
                Tile::Wall => '#',
                Tile::Floor => ' ',
                Tile::Door => '+',
            })
            .collect();
        for a in &self.agents {
            let action = match a.doing {
                Doing::Walking(act, _) | Doing::Busy(act, _) => Some(act),
                Doing::Idle => None,
            };
            grid[a.pos.y as usize * W + a.pos.x as usize] = match action {
                Some(Action::Eat) => 'E',
                Some(Action::Sleep) => 'S',
                Some(Action::Socialize) => 'O',
                Some(Action::Pray) => 'P',
                Some(Action::Wander) => 'W',
                Some(Action::Hoard) => 'H',
                Some(Action::CarryHome) => 'C',
                None => 'I',
            };
        }
        (0..H).map(|y| grid[y * W..(y + 1) * W].iter().collect::<String>() + "\n").collect()
    }

    pub fn summary(&self) -> String {
        let mut counts = [0; 7];
        for a in &self.agents {
            if let Doing::Walking(act, _) | Doing::Busy(act, _) = a.doing {
                counts[act as usize] += 1;
            }
        }
        let fear = self.agents.iter().map(|a| a.beliefs.fear).sum::<f32>() / self.agents.len() as f32;
        let secs = self.countdown_left() / TICKS_PER_SEC;
        format!(
            "tick {} | day {}/{} | countdown {}:{:02} | eat {} sleep {} socialize {} pray {} wander {} hoard {} | fear {:.2}",
            self.tick,
            (self.tick / self.preset.ticks_per_day() + 1).min(self.preset.shape().0),
            self.preset.shape().0,
            secs / 60,
            secs % 60,
            counts[0],
            counts[1],
            counts[2],
            counts[3],
            counts[4],
            counts[5] + counts[6],
            fear,
        )
    }
}

fn goal(world: &World, a: &Agent, action: Action, rng: &mut Rng) -> Pos {
    match action {
        Action::Eat => world.door_of(BuildingKind::Market),
        Action::Sleep => world.buildings[a.home].door,
        Action::Socialize => world.door_of(BuildingKind::Tavern),
        Action::Pray => world.door_of(BuildingKind::Chapel),
        Action::Wander => world.random_open(rng),
        Action::Hoard => world.door_of(BuildingKind::Market),
        Action::CarryHome => world.buildings[a.home].door,
    }
}

/// Sneaky: the hoarder only starts with at most `floor(urgency × 3)` agents in sight,
/// so early in the run nobody may be watching at all.
pub fn hoard_gate(watchers: usize, urgency: f32) -> bool {
    watchers <= (urgency * 3.0).floor() as usize
}

/// How long an action keeps the agent busy at its destination, in ticks.
fn duration(action: Action, day: f32) -> u64 {
    let frac = match action {
        Action::Eat => 1.0 / 24.0,
        Action::Sleep => 1.0 / 4.0,
        Action::Socialize => 1.0 / 12.0,
        Action::Pray => 1.0 / 16.0,
        Action::Wander => 0.0,
        Action::Hoard => 1.0 / 48.0,
        Action::CarryHome => 0.0,
    };
    (day * frac) as u64
}

/// Give `SPECIAL_STATIONS` to distinct random agents. Own rng stream, so the sim rng's draws
/// (needs, beliefs, movement) are unchanged. Rumor spread does change, since stations feed `influence`.
///
/// TODO: stations don't affect behavior yet. docs/rumor-engine.md wants the tavern keeper working
/// at the tavern and the prophet and priest favoring the chapel.
/// TODO(open question): religious stations draw beliefs like townsfolk, so a prophet can roll
/// loyalty near 0 (influence ~0.7) and rarely pray. Decide whether they get adjusted beliefs.
fn assign_stations(seed: u64, agents: &mut [Agent]) {
    let mut rng = Rng::new(seed ^ 0x53_5441_5449_4F4E); // "STATION"
    let mut ids: Vec<usize> = (0..agents.len()).collect();
    for station in SPECIAL_STATIONS {
        if ids.is_empty() {
            break;
        }
        let id = ids.swap_remove(rng.below(ids.len()));
        agents[id].station = station;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Station;

    fn stations(seed: u64, agent_count: usize) -> Vec<Station> {
        Sim::new(seed, Preset::Quick, agent_count).agents.iter().map(|a| a.station).collect()
    }

    #[test]
    fn stations_assigned_from_seed() {
        let s = stations(42, 50);
        let count = |st| s.iter().filter(|&&x| x == st).count();
        assert_eq!(count(Station::Prophet), 1);
        assert_eq!(count(Station::Priest), 1);
        assert_eq!(count(Station::TavernKeeper), 1);
        assert_eq!(count(Station::Elder), 2);
        assert_eq!(count(Station::Beggar), 3);
        assert_eq!(count(Station::Townsfolk), 42);

        assert_eq!(s, stations(42, 50));
        assert_ne!(s, stations(43, 50));

        // A town smaller than the special list fills from the front and doesn't panic.
        let mut small = stations(42, 3);
        small.sort_by_key(|st| *st as u8);
        assert_eq!(small, [Station::Prophet, Station::Priest, Station::TavernKeeper]);
    }

    #[test]
    fn hoard_gate_allows_more_eyes_as_urgency_rises() {
        assert!(hoard_gate(0, 0.0));
        assert!(!hoard_gate(1, 0.33), "floor(0.99) = 0: nobody may watch");
        assert!(hoard_gate(1, 0.34));
        assert!(hoard_gate(3, 1.0));
        assert!(!hoard_gate(4, 1.0));
    }

    #[test]
    fn only_the_hidden_hoarder_hoards_and_gets_sighted() {
        let mut sim = Sim::new(42, Preset::Quick, 50);
        let hoarder = sim.truth.hoarder.expect("seed 42 has a hoarder");
        let mut hoarded = false;
        for _ in 0..Preset::Quick.total_ticks() {
            sim.tick();
            for (id, a) in sim.agents.iter().enumerate() {
                if let Doing::Walking(act, _) | Doing::Busy(act, _) = a.doing {
                    if matches!(act, Action::Hoard | Action::CarryHome) {
                        assert_eq!(id, hoarder, "tick {}", sim.tick);
                        hoarded = true;
                    }
                }
            }
        }
        assert!(hoarded, "the hoarder hoards at least once in a whole run");
        let sightings = sim.rumors.claims.iter().filter(|c| c.origin == crate::rumor::Origin::Sighting);
        assert!(sightings.clone().count() > 0, "someone saw it");
        assert!(sightings.clone().all(|c| c.proposition == Proposition::Hoarder(hoarder)));
    }

    const TEN_MINUTES: u64 = 10 * 60 * TICKS_PER_SEC;

    fn run(seed: u64) -> String {
        let mut sim = Sim::new(seed, Preset::Quick, 50);
        for _ in 0..TEN_MINUTES {
            sim.tick();
        }
        format!("{sim:?}")
    }

    #[test]
    fn same_seed_same_final_state() {
        assert_eq!(run(42), run(42));
    }

    #[test]
    fn different_seeds_diverge() {
        assert_ne!(run(1), run(2));
    }
}
