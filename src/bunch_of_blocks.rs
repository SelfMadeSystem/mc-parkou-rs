use rand::{seq::SliceRandom, Rng};
use valence::prelude::*;

use crate::{game_state::GameState, parkour_gen_params::ParkourGenParams, BLOCK_TYPES};

/// A bunch of blocks that are spawned at once.
pub struct BunchOfBlocks {
    /// The blocks that are spawned.
    blocks: Vec<(BlockPos, BlockState)>,
    /// The gen params for the next bunch of blocks.
    pub next_params: ParkourGenParams,
    /// Type of the bunch.
    pub bunch_type: BunchType,
}

impl BunchOfBlocks {
    pub fn single(pos: BlockPos, block: BlockState, state: &GameState) -> Self {
        Self {
            blocks: vec![(pos, block)],
            next_params: ParkourGenParams::basic_jump(pos, state),
            bunch_type: BunchType::Single,
        }
    }

    pub fn island(pos: BlockPos, size: i32, state: &GameState) -> Self {
        let mut blocks = vec![];

        for z in 0..=size * 2 {
            // the min to max range of x values so that the island is a circle
            // Since z starts at 0 and goes to size * 2, we need to subtract size
            // to make it go from -size to size.
            let s = {
                let size = size as f32;
                let mut z = z as f32;
                if z == 0. {
                    z = 0.25;
                } else if z == size * 2. {
                    z = size * 2. - 0.25;
                }
                ((size * size - (z - size) * (z - size)) as f32)
                    .sqrt()
                    .round() as i32
            };
            for x in -s..=s {
                let pos = BlockPos {
                    x: pos.x + x,
                    y: pos.y,
                    z: pos.z + z,
                };
                blocks.push((
                    pos,
                    if x == 0 && z == 0 {
                        BlockState::STONE
                    } else {
                        BlockState::GRASS_BLOCK
                    },
                ));
            }
        }

        let end_pos = BlockPos {
            x: pos.x,
            y: pos.y,
            z: pos.z + size * 2,
        };

        Self {
            blocks,
            next_params: ParkourGenParams::basic_jump(end_pos, state),
            bunch_type: BunchType::Island,
        }
    }

    pub fn slime_jump(pos: BlockPos, state: &GameState) -> Self {
        let mut slime_pos = pos;
        let mut blocks = vec![];
        let mut rng = rand::thread_rng();

        let dist = match state.prev_type {
            Some(BunchType::SlimeJump) => rng.gen_range(3..6),
            _ => rng.gen_range(6..8),
        };
        println!("dist: {}", dist);
        slime_pos.z += dist;
        slime_pos.y -= 10;

        for x in -1..=1 {
            for z in -1..=1 {
                let pos = BlockPos {
                    x: slime_pos.x + x,
                    y: slime_pos.y,
                    z: slime_pos.z + z,
                };
                blocks.push((pos, BlockState::SLIME_BLOCK));
            }
        }

        Self {
            blocks,
            next_params: ParkourGenParams::exact(BlockPos {
                x: pos.x,
                y: pos.y - 5,
                z: pos.z
                    + dist
                    + match state.prev_type {
                        Some(BunchType::SlimeJump) => rng.gen_range(3..5),
                        _ => rng.gen_range(4..7),
                    }
                    - 1,
            }),
            bunch_type: BunchType::SlimeJump,
        }
    }

    pub fn place(&self, world: &mut ChunkLayer) {
        for (pos, block) in &self.blocks {
            world.set_block(*pos, *block);
        }

        // debug only
        world.set_block(
            self.next_params.end_pos.get_in_direction(Direction::Up),
            BlockState::RAIL,
        );
        world.set_block(
            self.next_params.next_pos.get_in_direction(Direction::Up),
            BlockState::REDSTONE_WIRE,
        );

        if self.next_params.end_pos == self.next_params.next_pos {
            world.set_block(
                self.next_params.end_pos.get_in_direction(Direction::Up),
                BlockState::GRASS,
            );
        }
    }

    pub fn remove(&self, world: &mut ChunkLayer) {
        for (pos, _) in &self.blocks {
            world.set_block(*pos, BlockState::AIR);
        }
    }

    /// Returns true if the player has reached any of the blocks.
    pub fn has_reached(&self, pos: Position) -> bool {
        let pos = BlockPos::new(
            (pos.0.x - 0.5).round() as i32,
            pos.0.y as i32 - 1,
            (pos.0.z - 0.5).round() as i32,
        );

        self.blocks.iter().any(|(block_pos, _)| *block_pos == pos)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BunchType {
    Single,
    Island,
    SlimeJump,
}

impl BunchType {
    pub fn generate(&self, params: &ParkourGenParams, state: &GameState) -> BunchOfBlocks {
        let mut rng = rand::thread_rng();

        match self {
            Self::Single => BunchOfBlocks::single(
                params.next_pos,
                *BLOCK_TYPES.choose(&mut rng).unwrap(),
                state,
            ),
            Self::Island => BunchOfBlocks::island(params.next_pos, rng.gen_range(1..5), state),
            Self::SlimeJump => BunchOfBlocks::slime_jump(params.end_pos, state),
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();

        match rng.gen_range(0..3) {
            0 => Self::Single,
            1 => Self::Island,
            _ => Self::SlimeJump,
        }
    }

    pub fn random_up() -> Self {
        let mut rng = rand::thread_rng();

        match rng.gen_range(0..2) {
            0 => Self::Single,
            _ => Self::Island,
        }
    }

    pub fn random_down() -> Self {
        let mut rng = rand::thread_rng();

        match rng.gen_range(0..2) {
            // 0 => Self::Single,
            _ => Self::SlimeJump,
        }
    }
}
