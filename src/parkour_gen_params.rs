use rand::Rng;
use valence::prelude::*;

use crate::{
    bunch_of_blocks::{BunchOfBlocks, BunchType},
    game_state::GameState,
    line::Line3,
    prediction::prediction_state::PredictionState,
};

/// The parameters to generate the next bunch of blocks.
pub struct ParkourGenParams {
    /// The position of the block to expect the player to be standing on when they reach the end of the previous bunch.
    pub end_pos: BlockPos,
    /// The position to expect the start of the next bunch of blocks.
    pub next_pos: BlockPos,
    /// The initial PlayerState to expect the player to be in when they reach the end of the previous bunch.
    pub initial_state: PredictionState,
    /// The final state to expect the player to be in when they reach the beginning of the next bunch.
    pub next_state: PredictionState,
    /// The lines stuffs idk i dont want to explain it
    pub lines: Vec<Line3>,
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
        let initial_state = PredictionState::running_jump(pos, random_yaw());

        let mut rng = rand::thread_rng();

        let ticks = match state.target_y {
            0 => rng.gen_range(8..=16),
            y if y > pos.y => rng.gen_range(4..=8),
            _ => rng.gen_range(5..=14),
        };
        let (next_state, lines) = initial_state.get_state_in_ticks(ticks);
        let mut next_pos = next_state.get_block_pos();

        if (next_pos.y - pos.y == 1 && next_pos.z - pos.z == 4)
            || (next_pos.y - pos.y == 0 && next_pos.z - pos.z == 5)
        {
            next_pos.z -= 1; // I don't want 4 block jumps or 3 forward 1 up jumps
        }

        Self {
            end_pos: pos,
            next_pos,
            initial_state,
            next_state,
            lines,
            ticks,
        }
    }

    pub fn fall(pos: BlockPos) -> Self {
        let initial_state = PredictionState::head_hit_jump(pos, random_yaw_dist(35.));

        let mut rng = rand::thread_rng();

        let ticks = rng.gen_range(4..=10);

        let (next_state, lines) = initial_state.get_state_in_ticks(ticks);

        Self {
            end_pos: pos,
            next_pos: next_state.get_block_pos(),
            initial_state,
            next_state,
            lines,
            ticks,
        }
    }

    pub fn bounce(state: PredictionState) -> Self {
        let mut pos = state.get_block_pos();
        let mut initial_state = state;
        let mut next_state: PredictionState;
        let mut new_pos: BlockPos;
        let mut ticks: u32;

        let mut rng = rand::thread_rng();

        let ydiff = rng.gen_range(1..=3);
        let ydiffmax = rng.gen_range(0..=1);

        if initial_state.yaw > 30. * std::f32::consts::PI / 180. {
            initial_state.yaw -= rng.gen_range(0. ..=15.) * std::f32::consts::PI / 180.;
        } else if initial_state.yaw < -30. * std::f32::consts::PI / 180. {
            initial_state.yaw += rng.gen_range(0. ..=15.) * std::f32::consts::PI / 180.;
        } else {
            initial_state.yaw += rng.gen_range(-15. ..=15.) * std::f32::consts::PI / 180.;
        }

        let mut lines = Vec::new();
        let mut prev = initial_state.pos.as_vec3();

        loop {
            while initial_state.pos.y > pos.y as f64 {
                initial_state.tick();

                let next = initial_state.pos.as_vec3();
                lines.push(Line3::new(prev, next));
                prev = next;
            }

            next_state = initial_state.clone();
            next_state.pos.y = pos.y as f64;
            next_state.vel.y *= -0.8;
            ticks = 0;

            let mut new_lines = Vec::new();
            let mut new_prev = next_state.pos.as_vec3();

            while next_state.vel.y > 0. {
                next_state.tick();
                ticks += 1;

                let next = next_state.pos.as_vec3();
                new_lines.push(Line3::new(new_prev, next));
                new_prev = next;
            }

            new_pos = next_state.get_block_pos();

            if new_pos.y - pos.y < ydiff + ydiffmax {
                pos.y -= 1;
                continue;
            }

            lines.extend(new_lines);
            prev = new_prev;

            let y = next_state.pos.y.floor();

            loop {
                let mut new_next_state = next_state.clone();
                new_next_state.tick();
                ticks += 1;

                let next = new_next_state.pos.as_vec3();
                lines.push(Line3::new(prev, next));
                prev = next;

                if new_next_state.pos.y > y - ydiffmax as f64 {
                    next_state = new_next_state;
                } else {
                    break;
                }
            }

            new_pos = next_state.get_block_pos();

            break;
        }

        initial_state.pos.y = pos.y as f64;
        initial_state.vel.y *= -0.8;

        Self {
            end_pos: initial_state.get_block_pos(),
            next_pos: new_pos,
            initial_state,
            next_state,
            lines,
            ticks,
        }
    }

    pub fn generate(&self, state: &GameState) -> BunchOfBlocks {
        BunchType::random(self.next_pos, state).generate(self, state)
    }
}
