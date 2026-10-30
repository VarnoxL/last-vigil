use crate::rng::Rng;
use crate::world::Pos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Eat,
    Sleep,
    Socialize,
    Pray,
    Wander,
}

#[derive(Debug, Clone)]
pub enum Doing {
    Idle,
    /// Remaining path, next step last.
    Walking(Action, Vec<Pos>),
    /// Ticks left at the destination.
    Busy(Action, u64),
}

/// 0 = satisfied, 1 = desperate.
#[derive(Debug, Clone)]
pub struct Needs {
    pub hunger: f32,
    pub fatigue: f32,
    pub loneliness: f32,
}

/// Each 0..1.
#[derive(Debug, Clone)]
pub struct Beliefs {
    /// Belief that the prophecy is true.
    pub faith: f32,
    pub fear: f32,
    /// Devotion to the cult itself, apart from the prophecy.
    pub loyalty: f32,
}

/// Social role; sets base influence in the rumor engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Station {
    Prophet,
    Priest,
    TavernKeeper,
    Elder,
    Townsfolk,
    Beggar,
}

impl Station {
    pub fn base(self) -> f32 {
        match self {
            Self::Prophet => 0.8,
            Self::Priest => 0.6,
            Self::TavernKeeper | Self::Elder => 0.5,
            Self::Townsfolk => 0.3,
            Self::Beggar => 0.1,
        }
    }
}

/// An agent's id is its index in `Sim::agents`.
#[derive(Debug, Clone)]
pub struct Agent {
    pub pos: Pos,
    pub station: Station,
    /// Index into `World::buildings`.
    pub home: usize,
    pub needs: Needs,
    pub beliefs: Beliefs,
    pub doing: Doing,
}

impl Agent {
    pub fn new(home: usize, pos: Pos, rng: &mut Rng) -> Self {
        let needs = Needs { hunger: rng.unit() * 0.5, fatigue: rng.unit() * 0.5, loneliness: rng.unit() * 0.5 };
        let faith = rng.unit();
        let beliefs = Beliefs { faith, fear: 0.0, loyalty: faith * rng.unit() };
        // TODO(phase 3): assign stations from the seed (1 prophet, 1 priest, 1 tavern keeper, 2 elders, 3 beggars).
        Self { pos, station: Station::Townsfolk, home, needs, beliefs, doing: Doing::Idle }
    }

    /// Utility AI: score each action from needs and beliefs and take the best.
    /// The jitter keeps the crowd from moving in lockstep.
    pub fn choose(&self, rng: &mut Rng) -> Action {
        let (n, b) = (&self.needs, &self.beliefs);
        [
            (Action::Eat, n.hunger),
            (Action::Sleep, n.fatigue),
            (Action::Socialize, n.loneliness * 0.8),
            (Action::Pray, b.faith * (0.2 + b.fear) * 0.8 + b.loyalty * 0.1),
            (Action::Wander, 0.2),
        ]
        .into_iter()
        .map(|(a, s)| (a, s + rng.unit() * 0.1))
        .max_by(|x, y| x.1.total_cmp(&y.1))
        .unwrap()
        .0
    }

    /// Applied when the agent finishes an action at its destination.
    pub fn finish(&mut self, action: Action) {
        match action {
            Action::Eat => self.needs.hunger = 0.0,
            Action::Sleep => self.needs.fatigue = 0.0,
            Action::Socialize => self.needs.loneliness = 0.0,
            Action::Pray => self.beliefs.fear *= 0.5,
            Action::Wander => {}
        }
    }
}
