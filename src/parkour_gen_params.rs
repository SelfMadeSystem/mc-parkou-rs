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
    /// The initial PlayerState to use when generating the next bunch of blocks.
    pub initial_state: PlayerState,
    pub t: u32,
}

fn random_yaw() -> f32 {
    rand::thread_rng().gen_range(-60.0..60.0) * std::f32::consts::PI / 180.0
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
        let mut new_state = initial_state.clone();
        let mut rng = rand::thread_rng();
        let mut t = 0;
        match state.target_y {
            0 => {
                for _ in 0..rng.gen_range(10..=20) {
                    new_state.tick();
                    t += 1;
                }
            },
            y if y > pos.y => {
                for _ in 0..rng.gen_range(4..=8) {
                    new_state.tick();
                    t += 1;
                }
            }
            _ => {
                for _ in 0..rng.gen_range(5..=15) {
                    new_state.tick();
                    t += 1;
                }
            }
        };
        let y = new_state.pos.y.floor() as i32 - pos.y - 1;
        let z = new_state.pos.z.floor() as i32 - pos.z;// - rng.gen_range(0..2);
        let x = new_state.pos.x as i32 - pos.x;
        Self {
            end_pos: pos,
            next_pos: BlockPos {
                x: pos.x + x,
                y: pos.y + y,
                z: pos.z + z,
            },
            initial_state,
            t,
        }
    }

    pub fn fall(pos: BlockPos) -> Self {
        let mut rng = rand::thread_rng();

        let z = rng.gen_range(1..4);
        let x = rng.gen_range(-3..=3);
        let y = -((z * z + x * x) as f32).sqrt() as i32 * 2;
        Self {
            end_pos: pos,
            next_pos: BlockPos {
                x: pos.x + x,
                y: pos.y + y,
                z: pos.z + z,
            },
            initial_state: PlayerState::head_hit_jump(pos, random_yaw()),
            t: 16,
        }
    }

    pub fn generate(&self, state: &GameState) -> BunchOfBlocks {
        BunchType::random(self.next_pos, state).generate(self, state)
    }
}
