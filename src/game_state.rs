use std::collections::VecDeque;

use valence::prelude::*;

use crate::{bunch_of_blocks::{BunchOfBlocks, BunchType}, prediction::player_state::PlayerState};

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
}