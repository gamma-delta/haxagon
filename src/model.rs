use std::collections::VecDeque;

use ahash::{AHashMap, AHashSet};
use enum_map::Enum;
use hex2d::{Angle, Coordinate, Direction, Spin};
use quad_rand::compat::QuadRand;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Board full of marbles to play on
#[derive(Debug)]
pub struct Board {
    marbles: AHashMap<Coordinate, Marble>,
    score: u32,

    action_queue: VecDeque<BoardAction>,
    /// Time counting up until we do the next action
    action_timer: u32,

    /// Count up until we spawn the next marble
    next_spawn_timer: u32,
    planned_next_spawn_pos: Option<Coordinate>,

    tick_count: u32,

    settings: BoardSettings,
}

impl Board {
    /// Create a new Board with the given size. There will be the given number of "rings"
    /// of marbles around the outside.
    pub fn new(settings: BoardSettings) -> Self {
        let pad = settings.radius - settings.border_width;
        let mut out = Board {
            marbles: AHashMap::new(),
            score: 0,
            action_queue: VecDeque::new(),
            action_timer: 0,
            next_spawn_timer: 0,

            // we're about to set this in
            planned_next_spawn_pos: Some(Coordinate::new(pad as i32, 0)),
            tick_count: 0,
            settings,
        };

        for dist in pad..=out.radius() {
            for c in Coordinate::new(0, 0).ring_iter(dist as i32 + 1, Spin::CW(Direction::XY)) {
                out.spawn_marble(&c);
            }
        }

        out
    }

    /// Run one frame of the board. Return `true` if we die.
    pub fn tick(&mut self) -> bool {
        self.next_spawn_timer += 1;
        if self.next_spawn_timer >= self.timer_max() {
            self.next_spawn_timer = 0;

            if let Some(sp) = self.planned_next_spawn_pos {
                self.spawn_marble(&sp);
                self.gravitate();
                self.action_queue.push_back(BoardAction::ClearBlobs(1));
                self.planned_next_spawn_pos = self.find_next_spawnpoint(sp);
            } else {
                // oh no we couldn't find a place to be
                return true;
            }
        }

        let do_action = loop {
            let do_action = match self.action_queue.front() {
                Some(it) => {
                    if let BoardAction::ClearBlobs(_) = it {
                        let blobs = self.find_blobs();
                        if blobs.is_empty() {
                            // Skip clearing blobs if we didn't find any blobs.
                            self.action_queue.pop_front();
                            continue;
                        }
                    }

                    self.action_timer += 1;
                    self.action_timer >= it.time()
                }
                _ => false,
            };
            break do_action;
        };
        if do_action {
            let action = self.action_queue.pop_front().unwrap();
            self.execute_action(action);
            self.action_timer = 0;
            self.gravitate();

            // This action likely moved some marbles, so let's reposition the spawnpoint
            if let Some(next_sp) = self.planned_next_spawn_pos {
                let shunted = self.gravity_all(next_sp);
                self.planned_next_spawn_pos = Some(shunted);
            }
        }

        self.tick_count += 1;

        false
    }

    /// Find all the blobs of marbles with size >= the given.
    pub fn find_blobs(&self) -> Vec<Vec<Coordinate>> {
        let mut seen = AHashSet::new();
        let mut out = Vec::new();
        for c in self.marbles.keys() {
            if !seen.insert(*c) {
                // we've seen this
                continue;
            }
            let blob = self.floodfill(c);
            seen.extend(blob.iter().copied());

            if blob.len() >= self.settings.clear_blob_size {
                out.push(blob)
            }
        }
        out
    }

    pub fn next_spawn_point(&self) -> Option<Coordinate> {
        self.planned_next_spawn_pos
    }

    /// Return if the coordinate lies within the board
    pub fn is_in_bounds(&self, c: &Coordinate) -> bool {
        c.distance(Coordinate::new(0, 0)) <= self.radius() as i32
    }

    /// The player has done a thing and the board needs to update
    pub fn push_action(&mut self, action: BoardAction) {
        self.action_queue.push_back(action);
    }

    /// The action we're going to execute.
    pub fn next_action(&self) -> Option<&BoardAction> {
        self.action_queue.front()
    }

    /// Get all the marbles in the board
    pub fn get_marbles(&self) -> &AHashMap<Coordinate, Marble> {
        &self.marbles
    }

    /// Helper function to get one marble
    pub fn get_marble(&self, pos: &Coordinate) -> Option<&Marble> {
        self.marbles.get(pos)
    }

    /// Get a reference to the board's action timer.
    pub fn action_timer(&self) -> u32 {
        self.action_timer
    }

    /// Get a reference to the board's next spawn timer.
    pub fn next_spawn_timer(&self) -> u32 {
        self.next_spawn_timer
    }

    /// Get a reference to the board's radius.
    pub fn radius(&self) -> usize {
        self.settings.radius
    }

    /// Get a reference to the board's settings.
    pub fn settings(&self) -> &BoardSettings {
        &self.settings
    }

    /// Get a reference to the board's score.
    pub fn score(&self) -> u32 {
        self.score
    }

    /// Get if a position is inside a marble or out of bounds
    pub fn is_solid(&self, c: &Coordinate) -> bool {
        !self.is_in_bounds(c) || self.get_marble(c).is_some()
    }

    /// If the previous spawnpoint was here, wehere is the next spawnpoint?
    fn find_next_spawnpoint(&self, prev: Coordinate) -> Option<Coordinate> {
        // clockwise iter
        let maybe_pos = (|| {
            for dir in Direction::all() {
                // Use a maze algorithm: always keep your left hand on the wall.
                let ahead = prev + *dir;
                let wallfinder = prev + (*dir + Angle::Left);

                if !self.is_solid(&ahead) && self.is_solid(&wallfinder) {
                    // here's our pos! but let's gravitate it to avoid jank
                    return Some(ahead);
                }
            }
            None
        })();
        let maybe_pos = match maybe_pos {
            Some(it) => Some(it),
            None => {
                // uh oh ... look for the closest empty spot
                Coordinate::new(0, 0)
                    .range_iter(self.radius() as i32)
                    .filter(|pos| self.get_marble(pos).is_none())
                    .min_by_key(|pos| pos.distance(prev))
            }
        };
        // Shunt the spawnpoint to the outside, even if there's no gravity.
        maybe_pos.map(|pos| self.gravity_all(pos))
    }

    fn timer_max(&self) -> u32 {
        let out = match self.tick_count {
            it if it < 60 * 10 => 60,
            it if it < 60 * 20 => 50,
            it if it < 60 * 40 => 40,
            it if it < 60 * 60 => 30,
            it if it < 60 * 120 => 40,
            it => 40u32.saturating_sub(it / (60 * 30)).max(20),
        };
        (out as f32 / self.settings.spawn_multiplier) as u32
    }

    /// Run the action on the board
    fn execute_action(&mut self, action: BoardAction) {
        match action {
            BoardAction::Cycle(poses) => {
                if poses.len() >= 2 {
                    // Swap in a reversed order to end up with rotation in the right order.
                    for pair in poses.windows(2).rev() {
                        let a = self.marbles.remove(&pair[0]);
                        let b = self.marbles.remove(&pair[1]);
                        if let Some(a) = a {
                            self.marbles.insert(pair[1], a);
                        }
                        if let Some(b) = b {
                            self.marbles.insert(pair[0], b);
                        }
                    }
                }
            }
            BoardAction::DeleteColor(color) => {
                let original_len = self.marbles.len();
                self.marbles.retain(|_, marble| marble != &color);
                self.score += (original_len - self.marbles.len()) as u32;
            }
            BoardAction::ClearBlobs(multiplier) => {
                let blobs = self.find_blobs();
                let blob_count = blobs.len();
                let to_remove = blobs.into_iter().flatten().collect::<Vec<_>>();
                if !to_remove.is_empty() {
                    // Each marble above 6 is counted double
                    let score = to_remove.len() + to_remove.len().saturating_sub(6);
                    self.score += score as u32 * multiplier * blob_count as u32;
                    // This might cause a cascade: immediately do another
                    self.action_queue
                        .push_front(BoardAction::ClearBlobs(multiplier + 1));
                    for c in to_remove {
                        self.marbles.remove(&c);
                    }
                }
            }
        }
    }

    fn gravitate(&mut self) {
        if self.settings.gravity {
            loop {
                let mut shunted_any = false;

                let poses = self.marbles.keys().cloned().collect::<Vec<_>>();
                for pos in poses {
                    let target = self.gravity_step(&pos);
                    if let Some(target) = target {
                        let m = self.marbles.remove(&pos).unwrap();
                        self.marbles.insert(target, m);
                        shunted_any = true;
                    }
                }

                if !shunted_any {
                    break;
                }
            }
        }
    }

    /// Find the place the coordinate falls to under gravity, or None if it doesn't.
    fn gravity_step(&self, c: &Coordinate) -> Option<Coordinate> {
        let gravity = c.direction_from_center_cw().unwrap_or(Direction::YX);

        let mut shunt = None;
        let mut solid_poses = 0;
        for angle in [Angle::Forward, Angle::Left, Angle::Right] {
            let dir = gravity + angle;

            let target = *c + dir;
            if self.is_in_bounds(&target) && !self.marbles.contains_key(&target) {
                // shunt the marble here!
                if shunt.is_none() {
                    shunt = Some(target);
                }
                // but keep going to record solid positions.
            } else {
                solid_poses += 1;
            }
        }

        // If there's enough solidity around DON'T FALL
        if solid_poses < 2 {
            shunt
        } else {
            None
        }
    }

    /// Repeatedly apply gravity to this point and return where it moves to.
    fn gravity_all(&self, mut c: Coordinate) -> Coordinate {
        while let Some(newpos) = self.gravity_step(&c) {
            c = newpos
        }
        c
    }

    /// Get all coordinates connected by color to the given coordinate (ignoring None)
    fn floodfill(&self, c: &Coordinate) -> Vec<Coordinate> {
        let color = match self.get_marble(c) {
            Some(it) => it,
            None => return Vec::new(),
        };

        let mut seen = AHashSet::new();
        let mut todo = vec![*c];
        let mut blob = Vec::new();
        while let Some(c) = todo.pop() {
            if !seen.contains(&c) && Some(color) == self.get_marble(&c) {
                seen.insert(c);
                todo.push(c);
                blob.push(c);
                todo.extend_from_slice(&c.neighbors());
            }
        }
        blob
    }

    /// Spawn a new random marble at the given position. Won't clobber existing marbles
    /// or form blobs big enough to score.
    /// Return `false` if it can't do it.
    fn spawn_marble(&mut self, c: &Coordinate) -> bool {
        if !self.is_in_bounds(c) || self.marbles.contains_key(c) {
            return false;
        }

        let mut marble = Marble::random(self.settings.marble_color_count);
        loop {
            self.marbles.insert(*c, marble.clone());
            if self.floodfill(c).len() < self.settings.clear_blob_size {
                // no overflow here!
                return true;
            }
            // There are 7 marble colors and only 6 possible neighbors,
            // so something will always happen eventually
            marble = marble.another();
        }
    }
}

/// Pieces that go on the board.
/// This is purposely *not* `Copy` to hopefully cut down on duplication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Marble {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Purple,
    Pink,
}

impl Marble {
    /// Make a random marble.
    pub fn random(max: usize) -> Self {
        use Marble::*;
        match QuadRand.gen_range(0..max.min(Marble::Pink as usize)) {
            0 => Red,
            1 => Green,
            2 => Blue,
            3 => Yellow,
            4 => Cyan,
            5 => Purple,
            6 => Pink,
            _ => panic!(),
        }
    }

    /// Give another color that isn't this one, for use after random generation
    /// doesn't go right.
    fn another(&self) -> Self {
        use Marble::*;
        match self {
            Red => Green,
            Green => Blue,
            Blue => Yellow,
            Yellow => Cyan,
            Cyan => Purple,
            Purple => Pink,
            Pink => Red,
        }
    }
}

/// Abstract actions that can happen on the board.
///
/// There's a bunch of variants here so I can experiment with gameplay stuff
#[derive(Debug, Clone)]
pub enum BoardAction {
    /// Shunt all the marbles on the coords along to the next coordinate
    ///
    /// DO NOT make the last the same as the first, this cycles it itself
    Cycle(Vec<Coordinate>),
    /// Delete all marbles of the given color
    DeleteColor(Marble),
    /// Clear all the large enough blobs of marbles, with the given score multiplier
    ClearBlobs(u32),
}

impl BoardAction {
    pub const CYCLE_TIME: u32 = 10;
    pub const DELETE_COLOR_TIME: u32 = 30;
    pub const CLEAR_BLOBS_TIME: u32 = 20;

    /// How many frames should it take to finish this action?
    pub fn time(&self) -> u32 {
        match self {
            BoardAction::Cycle(_) => Self::CYCLE_TIME,
            BoardAction::DeleteColor(_) => Self::DELETE_COLOR_TIME,
            BoardAction::ClearBlobs(_) => Self::CLEAR_BLOBS_TIME,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoardSettings {
    /// How many marbles to the edge from the center.
    /// (Radius of 0 is 1 marble)
    pub radius: usize,
    /// How many outside layers of marble to start
    pub border_width: usize,
    /// Whether gravity is on (it will point to the outside)
    pub gravity: bool,
    /// How many marbles need to be next to each other to clear
    pub clear_blob_size: usize,
    /// Multiplier on marble spawn rate
    pub spawn_multiplier: f32,
    /// How many colors of marbles try to spawn
    pub marble_color_count: usize,

    /// A key associated with this gamemode for storing scores, or None
    /// if it's a custom mode.
    pub mode_key: Option<BoardSettingsModeKey>,
}

impl BoardSettings {
    pub fn classic() -> Self {
        Self {
            radius: 5,
            border_width: 2,
            spawn_multiplier: 1.0,
            gravity: true,
            clear_blob_size: 4,
            marble_color_count: 6,
            mode_key: Some(BoardSettingsModeKey::Classic),
        }
    }

    pub fn advanced() -> Self {
        Self {
            radius: 6,
            border_width: 3,
            spawn_multiplier: 1.2,
            gravity: true,
            clear_blob_size: 4,
            marble_color_count: 7,
            mode_key: Some(BoardSettingsModeKey::Advanced),
        }
    }

    pub fn no_gravity() -> Self {
        Self {
            radius: 3,
            border_width: 2,
            spawn_multiplier: 0.8,
            gravity: false,
            clear_blob_size: 4,
            marble_color_count: 4,
            mode_key: Some(BoardSettingsModeKey::NoGravity),
        }
    }
}

#[non_exhaustive]
#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoardSettingsModeKey {
    Classic,
    Advanced,
    NoGravity,
}
