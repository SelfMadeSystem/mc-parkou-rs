use std::collections::{HashMap, HashSet, VecDeque};

use valence::prelude::*;

use crate::{
    bunch_of_blocks::{BunchOfBlocks, BunchType},
    line::Line3,
    prediction::player_state::PlayerState,
};

#[derive(Component)]
pub struct GameState {
    pub blocks: VecDeque<BunchOfBlocks>,
    pub prev_type: Option<BunchType>,
    pub score: u32,
    pub combo: u32,
    pub target_y: i32,
    pub stopped_running: bool,
    pub prev_pos: DVec3,
    pub test_state: PlayerState,
    pub line_entities: HashMap<Line3, Entity>,
    pub lines: HashSet<Line3>,
}
