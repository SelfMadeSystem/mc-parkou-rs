use rand::Rng;
use valence::prelude::*;

use crate::{
    bunch_of_blocks::{BunchOfBlocks, BunchType},
    game_state::GameState,
    prediction::player_state::PlayerState,
};

/// The parameters to generate the nex bunch of blocks.
pub struct ParkourGenParams {
    /// The position of the block to expect the player to be standing on when they reach the end of the previous bunch.
    pub end_pos: BlockPos,
    /// The position to expect the start of the next bunch of blocks.
    pub next_pos: BlockPos,
    /// The initial PlayerState to expect the player to be in when they reach the end of the previous bunch.
    pub initial_state: PlayerState,
    /// The final state to expect the player to be in when they reach the beginning of the next bunch.
    pub next_state: PlayerState,
    /// The number of ticks to expect the player to take to get from the end of the previous bunch to the beginning of the next bunch.
    pub ticks: u32,
}

fn random_yaw() -> f32 {
    random_yaw_dist(60.0)
}

fn random_yaw_dist(f: impl Into<f32>) -> f32 {
    let f = f.into();
    rand::thread_rng().gen_range(-f..f) * std::f32::consts::PI / 180.0
}

impl ParkourGenParams {
    // pub fn exact(pos: BlockPos) -> Self {
    //     Self {
    //         end_pos: pos,
    //         next_pos: pos,
    //     }
    // }

    pub fn basic_jump(pos: BlockPos, state: &GameState) -> Self {
        let initial_state = PlayerState::running_jump(pos, random_yaw());

        let mut rng = rand::thread_rng();
        
        let ticks = match state.target_y {
            0 =>rng.gen_range(8..=16),
            y if y > pos.y => rng.gen_range(4..=8),
            _ => rng.gen_range(5..=14),
        };
        let next_state = initial_state.get_state_in_ticks(ticks);

        Self {
            end_pos: pos,
            next_pos: next_state.get_block_pos(),
            initial_state,
            next_state,
            ticks,
        }
    }

    pub fn fall(pos: BlockPos) -> Self {
        let initial_state = PlayerState::head_hit_jump(pos, random_yaw_dist(35.));

        let mut rng = rand::thread_rng();

        let ticks = rng.gen_range(4..=10);

        let next_state = initial_state.get_state_in_ticks(ticks);

        Self {
            end_pos: pos,
            next_pos: next_state.get_block_pos(),
            initial_state,
            next_state,
            ticks: 16,
        }
    }

    pub fn generate(&self, state: &GameState) -> BunchOfBlocks {
        BunchType::random(self.next_pos, state).generate(self, state)
    }
}
