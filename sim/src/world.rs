use std::collections::VecDeque;

use crate::rng::Rng;

pub const W: usize = 64;
pub const H: usize = 48;
const ROAD_X: usize = 32;
const ROAD_Y: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Grass,
    Road,
    Wall,
    Floor,
    Door,
}

impl Tile {
    pub fn walkable(self) -> bool {
        self != Tile::Wall
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingKind {
    Home,
    Chapel,
    Tavern,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub x: u16,
    pub y: u16,
}

impl Pos {
    fn idx(self) -> usize {
        self.y as usize * W + self.x as usize
    }

    fn from_idx(i: usize) -> Self {
        Pos { x: (i % W) as u16, y: (i / W) as u16 }
    }
}

#[derive(Debug, Clone)]
pub struct Building {
    pub kind: BuildingKind,
    pub door: Pos,
}

#[derive(Debug, Clone)]
pub struct World {
    pub tiles: Vec<Tile>,
    pub buildings: Vec<Building>,
}

impl World {
    /// The one fixed town layout: two crossing roads, a chapel at the north end,
    /// tavern and market south of the crossing, and two rows of homes.
    pub fn town() -> Self {
        use BuildingKind::*;
        let mut tiles = vec![Tile::Grass; W * H];
        for x in 0..W {
            tiles[ROAD_Y * W + x] = Tile::Road;
        }
        for y in 0..H {
            tiles[y * W + ROAD_X] = Tile::Road;
        }

        // (kind, x, y, w, h)
        let mut layout = vec![(Chapel, 28, 4, 9, 7), (Tavern, 36, 26, 8, 6), (Market, 20, 26, 8, 6)];
        for x in [2, 8, 14, 20, 38, 44, 50, 56] {
            layout.push((Home, x, 18, 5, 4));
            layout.push((Home, x, 34, 5, 4));
        }

        let mut buildings = Vec::new();
        for (kind, x, y, w, h) in layout {
            for yy in y..y + h {
                for xx in x..x + w {
                    let edge = xx == x || xx == x + w - 1 || yy == y || yy == y + h - 1;
                    tiles[yy * W + xx] = if edge { Tile::Wall } else { Tile::Floor };
                }
            }
            // Door in the middle of the wall that faces the main road.
            let door_y = if y + h <= ROAD_Y { y + h - 1 } else { y };
            let door = Pos { x: (x + w / 2) as u16, y: door_y as u16 };
            tiles[door.idx()] = Tile::Door;
            buildings.push(Building { kind, door });
        }
        Self { tiles, buildings }
    }

    /// Door of the first building of this kind. Only homes come in multiples.
    pub fn door_of(&self, kind: BuildingKind) -> Pos {
        self.buildings.iter().find(|b| b.kind == kind).expect("town has one of each").door
    }

    /// A random outdoor tile.
    pub fn random_open(&self, rng: &mut Rng) -> Pos {
        loop {
            let i = rng.below(W * H);
            if matches!(self.tiles[i], Tile::Grass | Tile::Road) {
                return Pos::from_idx(i);
            }
        }
    }

    /// Shortest path, excluding `from`, with the next step *last* so callers can `pop()`.
    /// BFS is enough on a uniform-cost grid; switch to A* if tiles get movement costs.
    pub fn path(&self, from: Pos, to: Pos) -> Option<Vec<Pos>> {
        let (start, goal) = (from.idx(), to.idx());
        let mut prev = vec![usize::MAX; W * H];
        prev[start] = start;
        let mut queue = VecDeque::from([start]);
        while let Some(i) = queue.pop_front() {
            if i == goal {
                let mut path = Vec::new();
                let mut j = goal;
                while j != start {
                    path.push(Pos::from_idx(j));
                    j = prev[j];
                }
                return Some(path);
            }
            let (x, y) = (i % W, i / W);
            // Fixed neighbor order keeps paths deterministic.
            for (nx, ny) in [(x, y.wrapping_sub(1)), (x + 1, y), (x, y + 1), (x.wrapping_sub(1), y)] {
                if nx < W && ny < H {
                    let n = ny * W + nx;
                    if prev[n] == usize::MAX && self.tiles[n].walkable() {
                        prev[n] = i;
                        queue.push_back(n);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_door_reachable() {
        let world = World::town();
        let from = world.buildings[0].door;
        for b in &world.buildings {
            assert!(world.path(from, b.door).is_some(), "{:?} door unreachable", b.kind);
        }
    }
}
