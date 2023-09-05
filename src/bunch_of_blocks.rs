use noise::{utils::*, Fbm, SuperSimplex};
use rand::{seq::SliceRandom, Rng};
use valence::{layer::chunk::IntoBlock, prelude::*};

use crate::{block_types::*, game_state::GameState, parkour_gen_params::ParkourGenParams, prediction::player_state::PlayerState};

/// A bunch of blocks that are spawned at once.
pub struct BunchOfBlocks {
    /// The blocks that are spawned.
    blocks: Vec<(BlockPos, Block)>,
    /// The gen params for the next bunch of blocks.
    pub next_params: ParkourGenParams,
    /// Type of the bunch.
    pub bunch_type: BunchType,
}

impl BunchOfBlocks {
    /// Creates a single block jump.
    pub fn single(pos: BlockPos, block: impl IntoBlock, state: &GameState) -> Self {
        Self {
            blocks: vec![(pos, block.into_block())],
            next_params: ParkourGenParams::basic_jump(pos, state),
            bunch_type: BunchType::Single,
        }
    }

    // Creates an island of blocks. Right now, it's just a circle.
    pub fn island(pos: BlockPos, size: i32, state: &GameState) -> Self {
        fn get_x_radius(size: i32, z: i32) -> i32 {
            let size = size as f32;
            let mut z = z as f32;
            if z == -size {
                z = 0.25 - size;
            } else if z == size {
                z = size - 0.25;
            }
            (size * size - z * z).sqrt().round() as i32
        }

        fn get_dist_to_center(x: i32, z: i32) -> f32 {
            let x = x as f32;
            let z = z as f32;
            (x * x + z * z).sqrt()
        }

        let mut blocks = vec![];

        let mut rng = rand::thread_rng();

        let mut fbm = Fbm::<SuperSimplex>::new(rng.gen());
        fbm.octaves = 4;
        fbm.frequency = 0.5;
        fbm.persistence = 0.5;
        fbm.lacunarity = 2.;
        let a = size as f64 / 10.;
        let map: NoiseMap = PlaneMapBuilder::<_, 2>::new(&fbm)
            .set_size((size * 2 + 1) as usize, (size * 2 + 1) as usize)
            .set_x_bounds(-a, a)
            .set_y_bounds(-a, a)
            .set_is_seamless(true)
            .build();

        fn get_height(map: &NoiseMap, x: i32, z: i32) -> i32 {
            (map.get_value(z as usize, (x + map.size().1 as i32 / 2) as usize) * 5.0).round() as i32
        }

        let (avg_end_height, (max_end_height, max_end_height_x), (min_start, min_start_x)) = {
            let mut sum = 0;
            let mut max = i32::MIN;
            let mut max_x = 0i32;

            let mut min = i32::MAX;
            let mut min_x = 0i32;

            let z = size * 2;

            let s = get_x_radius(size, z - size);

            for x in -s..=s {
                {
                    let y = get_height(&map, x, z);

                    sum += y;

                    if y > max || (y == max && x.abs() < max_x.abs()) {
                        max = y;
                        max_x = x;
                    }
                }

                {
                    let y = get_height(&map, x, 0);

                    if y < min || (y == min && x.abs() < min_x.abs()) {
                        min = y;
                        min_x = x;
                    }
                }
            }

            (sum / (s * 2 + 1), (max, max_x), (min, min_x))
        };

        let pos = BlockPos {
            x: pos.x - min_start_x,
            y: pos.y - min_start,
            z: pos.z,
        };

        let mut min_y = i32::MAX;

        for z in 0..=size * 2 {
            let s = get_x_radius(size, z - size);
            for x in -s..=s {
                let y = get_height(&map, x, z);

                let pos = BlockPos {
                    x: pos.x + x,
                    y: pos.y + y,
                    z: pos.z + z,
                };

                if y < avg_end_height {
                    for y in 1..=(avg_end_height - y) {
                        let pos = BlockPos {
                            x: pos.x,
                            y: pos.y + y,
                            z: pos.z,
                        };
                        blocks.push((pos, BlockState::WATER.into_block()));
                    }

                    blocks.push((pos, BlockState::DIRT.into_block()));
                } else {
                    blocks.push((pos, BlockState::GRASS_BLOCK.into_block()));
                }

                {
                    let pos = BlockPos {
                        x: pos.x,
                        y: pos.y - 1,
                        z: pos.z,
                    };
                    blocks.push((pos, BlockState::DIRT.into_block()));
                }

                if y < min_y {
                    min_y = y;
                }
            }
        }

        let pow = rng.gen_range(1f32..1.75f32);

        for z in 0..=size * 2 {
            let s = get_x_radius(size, z - size);
            for x in -s..=s {
                let dist = get_dist_to_center(x, z - size);

                let y = get_height(&map, x, z);

                let mut down_to = min_y - (size as f32 - dist + 1.).powf(pow).round() as i32;

                down_to += (((y - min_y) as f32) * (dist / size as f32).powf(2.)) as i32;

                if down_to < y {
                    for y in down_to..y {
                        let pos = BlockPos {
                            x: pos.x + x,
                            y: pos.y + y - 1,
                            z: pos.z + z,
                        };
                        blocks.push((
                            pos,
                            UNDERGROUND_BLOCK_TYPES
                                .choose(&mut rng)
                                .unwrap()
                                .into_block(),
                        ));
                    }
                }
            }
        }

        let end_pos = BlockPos {
            x: pos.x + max_end_height_x,
            y: pos.y + max_end_height,
            z: pos.z + size * 2,
        };

        Self {
            blocks,
            next_params: ParkourGenParams::basic_jump(end_pos, state),
            bunch_type: BunchType::Island,
        }
    }

    /// Creates a head jump.
    pub fn head_jump(mut pos: BlockPos, state: &GameState) -> Self {
        let mut rng = rand::thread_rng();

        let mut blocks = vec![];
        let block_type = *GENERIC_BLOCK_TYPES.choose(&mut rng).unwrap();

        if matches!(state.prev_type, Some(BunchType::HeadJump)) {
            pos.z += 2;

            blocks.push((
                pos,
                block_type.into_block(),
            ));
        }

        for x in -2..=2 {
            blocks.push((
                BlockPos {
                    x: pos.x + x,
                    y: pos.y + 3,
                    z: pos.z + 1,
                },
                block_type.into_block(),
            ));
        }

        return Self {
            blocks,
            next_params: ParkourGenParams::fall(pos),
            bunch_type: BunchType::HeadJump,
        };
    }

    /// Creates a little ramp with slabs.
    pub fn run_up(pos: BlockPos, state: &GameState, length: i32) -> Self {
        let (block, slab) = *SLAB_TYPES.choose(&mut rand::thread_rng()).unwrap();

        let mut blocks = vec![];

        for x in -1..=1 {
            for y in 0..length {
                blocks.push((
                    BlockPos {
                        x: pos.x + x,
                        y: pos.y + y,
                        z: pos.z + y * 2,
                    },
                    block.into_block(),
                ));

                if y > 0 {
                    blocks.push((
                        BlockPos {
                            x: pos.x + x,
                            y: pos.y + y,
                            z: pos.z + y * 2 - 1,
                        },
                        slab.into_block(),
                    ));
                    blocks.push((
                        BlockPos {
                            x: pos.x + x,
                            y: pos.y + y - 1,
                            z: pos.z + y * 2 - 1,
                        },
                        slab.set(PropName::Type, PropValue::Top).into_block(),
                    ));
                }
            }
        }

        let end_pos = BlockPos {
            x: pos.x,
            y: pos.y + length - 1,
            z: pos.z + length * 2 - 2,
        };

        Self {
            blocks,
            next_params: ParkourGenParams::basic_jump(end_pos, state),
            bunch_type: BunchType::RunUp,
        }
    }

    /// Creates a slime block jump.
    pub fn slime_jump(initial_pos: BlockPos, prev_state: &PlayerState, _state: &GameState) -> Self {
        let next_params = ParkourGenParams::bounce(*prev_state, initial_pos);
        let pos = next_params.end_pos;
        let mut blocks = vec![];

        for x in -1..=1 {
            for z in -1..=1 {
                blocks.push((
                    BlockPos {
                        x: pos.x + x,
                        y: pos.y,
                        z: pos.z + z,
                    },
                    BlockState::SLIME_BLOCK.into_block(),
                ));
            }
        }

        Self {
            blocks,
            next_params,
            bunch_type: BunchType::SlimeJump,
        }
    }

    pub fn place(&self, world: &mut ChunkLayer) {
        for (pos, block) in &self.blocks {
            world.set_block(*pos, block.clone());
        }

        // debug only
        // world.set_block(
        //     self.next_params.end_pos.get_in_direction(Direction::Up),
        //     BlockState::RAIL.into_block(),
        // );
        // world.set_block(
        //     self.next_params.next_pos.get_in_direction(Direction::Up),
        //     BlockState::REDSTONE_WIRE.into_block(),
        // );

        // if self.next_params.end_pos == self.next_params.next_pos {
        //     world.set_block(
        //         self.next_params.end_pos.get_in_direction(Direction::Up),
        //         BlockState::GRASS.into_block(),
        //     );
        // }
    }

    pub fn remove(&self, world: &mut ChunkLayer) {
        for (pos, _) in &self.blocks {
            world.set_block(*pos, BlockState::AIR.into_block());
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
    HeadJump,
    RunUp,
    SlimeJump,
}

impl BunchType {
    pub fn generate(&self, params: &ParkourGenParams, state: &GameState) -> BunchOfBlocks {
        let mut rng = rand::thread_rng();

        match self {
            Self::Single => BunchOfBlocks::single(
                params.next_pos,
                *GENERIC_BLOCK_TYPES.choose(&mut rng).unwrap(),
                state,
            ),
            Self::Island => BunchOfBlocks::island(params.next_pos, rng.gen_range(2..8), state),
            Self::HeadJump => BunchOfBlocks::head_jump(params.end_pos, state),
            Self::RunUp => BunchOfBlocks::run_up(params.next_pos, state, rng.gen_range(2..5)),
            Self::SlimeJump => BunchOfBlocks::slime_jump(params.end_pos, &params.next_state, state),
        }
    }

    pub fn random(pos: BlockPos, state: &GameState) -> Self {
        match state.target_y {
            0 => BunchType::random_any(state),
            y if y > pos.y => BunchType::random_up(),
            _ => BunchType::random_down(state),
        }
    }

    pub fn random_any(_state: &GameState) -> Self {
        let mut rng = rand::thread_rng();

        match rng.gen_range(0..5) {
            0 => Self::Single,
            1 => Self::Island,
            2 => Self::HeadJump,
            3 => Self::RunUp,
            _ => Self::SlimeJump,
        }
    }

    pub fn random_up() -> Self {
        let mut rng = rand::thread_rng();

        match rng.gen_range(0..2) {
            0 => Self::Single,
            _ => Self::RunUp,
        }
    }

    pub fn random_down(_state: &GameState) -> Self {
        let mut rng = rand::thread_rng();

        match rng.gen_range(0..3) {
            0 => Self::HeadJump,
            1 => Self::Single,
            _ => Self::SlimeJump,
        }
    }
}
