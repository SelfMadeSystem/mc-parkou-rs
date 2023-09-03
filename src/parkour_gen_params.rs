use rand::Rng;
use valence::prelude::*;

use crate::{
    bunch_of_blocks::{BunchOfBlocks, BunchType},
    game_state::GameState,
};

/// The parameters to generate the nex bunch of blocks.
pub struct ParkourGenParams {
    /// The position of the block to expect the player to be standing on when they reach the end of the previous bunch.
    pub end_pos: BlockPos,
    /// The position to expect the start of the next bunch of blocks.
    pub next_pos: BlockPos,
}

impl ParkourGenParams {
    pub fn exact(pos: BlockPos) -> Self {
        Self {
            end_pos: pos,
            next_pos: pos,
        }
    }

    pub fn basic_jump(pos: BlockPos, state: &GameState) -> Self {
        let mut rng = rand::thread_rng();

        let y = match state.target_y {
            0 => rng.gen_range(-1..2),
            y if y > pos.y => 1,
            _ => rng.gen_range(-3..0),
        };
        let z = match y {
            1 => rng.gen_range(1..3),
            y if y < 0 => rng.gen_range(1..4) - (y - 1) / 2,
            _ => rng.gen_range(1..4),
        };
        let x = rng.gen_range(-3..4);
        Self {
            end_pos: pos,
            next_pos: BlockPos {
                x: pos.x + x,
                y: pos.y + y,
                z: pos.z + z,
            },
        }
    }

    pub fn generate(&self, state: &GameState) -> BunchOfBlocks {
        BunchType::random(self.next_pos, state).generate(self, state)
    }
}
