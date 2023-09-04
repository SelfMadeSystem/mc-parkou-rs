use std::collections::VecDeque;

use valence::prelude::*;

use crate::bunch_of_blocks::{BunchOfBlocks, BunchType};

#[derive(Component)]
pub struct GameState {
    pub blocks: VecDeque<BunchOfBlocks>,
    pub prev_type: Option<BunchType>,
    pub score: u32,
    pub combo: u32,
    pub target_y: i32,
    pub stopped_running: bool,
}