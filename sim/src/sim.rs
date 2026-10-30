use crate::agent::{Action, Agent, Doing};
use crate::rng::Rng;
use crate::rumor::{ClaimId, Proposition, RumorEngine};
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
        let agents = (0..agent_count)
            .map(|id| {
                let home = homes[id % homes.len()];
                Agent::new(home, world.buildings[home].door, &mut rng)
            })
            .collect();
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
        let Sim { agents, world, rng, .. } = self;

        // Fixed agent order: every rng draw happens in the same sequence each run.
        for a in agents.iter_mut() {
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
                    let action = a.choose(rng);
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
                Doing::Busy(action, 0) => {
                    a.finish(action);
                    Doing::Idle
                }
                Doing::Busy(action, left) => Doing::Busy(action, left - 1),
            };
        }

        // TODO(phase 3): once implemented, every `rumor::CHECK_EVERY` ticks run
        // `self.rumors.hearing_check(...)`, `record_sightings(...)`, and `drift_earned(...)`.
    }

    /// Text frame of the town: `#` wall, `+` door, `=` road, `.` grass, and each agent
    /// as the letter of what it's doing (Eat, Sleep, sOcialize, Pray, Wander, Idle).
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
                None => 'I',
            };
        }
        (0..H).map(|y| grid[y * W..(y + 1) * W].iter().collect::<String>() + "\n").collect()
    }

    pub fn summary(&self) -> String {
        let mut counts = [0; 5];
        for a in &self.agents {
            if let Doing::Walking(act, _) | Doing::Busy(act, _) = a.doing {
                counts[act as usize] += 1;
            }
        }
        let fear = self.agents.iter().map(|a| a.beliefs.fear).sum::<f32>() / self.agents.len() as f32;
        let secs = self.countdown_left() / TICKS_PER_SEC;
        format!(
            "tick {} | day {}/{} | countdown {}:{:02} | eat {} sleep {} socialize {} pray {} wander {} | fear {:.2}",
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
    }
}

/// How long an action keeps the agent busy at its destination, in ticks.
fn duration(action: Action, day: f32) -> u64 {
    let frac = match action {
        Action::Eat => 1.0 / 24.0,
        Action::Sleep => 1.0 / 4.0,
        Action::Socialize => 1.0 / 12.0,
        Action::Pray => 1.0 / 16.0,
        Action::Wander => 0.0,
    };
    (day * frac) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

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
