use std::collections::HashMap;

use crate::{alt_block::*, line::Line3, prediction::prediction_state::PredictionState, utils::*};

use super::{
    block_collection::*,
    custom_generation::{MultiCustomPreset, SingleCustomPreset},
    generation::*,
    generators::*,
    theme::GenerationTheme,
};
use rand::Rng;
use valence::{math::IVec2, prelude::*};

pub struct GenerateResult {
    pub start: BlockPos,
    pub end: BlockPos,
    pub blocks: HashMap<BlockPos, BlockState>,
    pub alt_blocks: HashMap<BlockPos, AltBlock>,
    pub lines: Vec<Line3>,
    pub children: Vec<ChildGeneration>,
}

impl GenerateResult {
    pub fn just_blocks(
        blocks: HashMap<BlockPos, BlockState>,
        start: BlockPos,
        end: BlockPos,
    ) -> Self {
        Self {
            start,
            end,
            blocks,
            alt_blocks: HashMap::new(),
            lines: Vec::new(),
            children: Vec::new(),
        }
    }
}

/// The `GenerationType` enum represents the different types of parkour generations
/// that can be used.
///
/// Variants:
/// * `Single`: The `Single` variant represents a single block.
/// * `Slime`: The `Slime` variant represents a slime block. // TODO
/// * `Ramp`: The `Ramp` variant represents blocks and slabs that are used to create
/// a ramp.
/// * `Island`: The `Island` variant represents blocks that are used to create an
/// island. // TODO
/// * `Indoor`: The `Indoor` variant represents blocks that are used to create an
/// indoor area.
/// * `Cave`: The `Cave` variant represents blocks that are used to create a cave.
/// * `Snake`: The `Snake` variant represents blocks that are used to create a
/// snake.
/// * `BlinkBlocks`: The `BlinkBlocks` variant represents blocks that are used to
/// create a blinking platform.
/// * `SingleCustom`: The `SingleCustom` variant represents a custom parkour
/// generation. It has preset blocks, a start position, and an end position.
/// * `MultiCustom`: The `MultiCustom` variant represents a custom parkour
/// generation. It has a start custom generation, a number of middle custom
/// generations, and an end custom generation. // TODO: Add examples
#[derive(Clone, Debug)]
pub enum GenerationType {
    Single(String),
    // Slime,
    Ramp(String),
    Island {
        grass: String,
        dirt: String,
        stone: String,
        water: String,
        min_radius: i32,
        max_radius: i32,
        min_point_power: f32,
        max_point_power: f32,
    },
    Indoor {
        /// Used for walls and ceiling
        walls: String,
        /// Used for floor. If `None`, there is no floor. If liquid, then `walls`
        /// is placed below `floor`.
        floor: Option<String>,
        /// Used for platforms
        platforms: String,
    },
    Cave(String),
    Snake(String),
    BlinkBlocks {
        on: String,
        off: String,
        delay: usize,
        overlap: usize,
    },
    SingleCustom(SingleCustomPreset),
    MultiCustom(MultiCustomPreset),
}

/// The `Generator` struct represents a parkour generator.
///
/// Properties:
///
/// * `theme`: The `theme` property is of type `GenerationTheme`. It represents the
/// theme of the parkour generator.
/// * `type`: The `type` property is of type `GenerationType`. It represents the
/// type of parkour generation that is used.
/// * `start`: The `start` property is of type `BlockPos`. It represents the start
/// position of the parkour generation.
#[derive(Clone, Debug)]
pub struct Generator {
    pub theme: GenerationTheme,
    pub generation_type: GenerationType,
    pub start: BlockPos,
}

impl Generator {
    pub fn first_in_generation(start: BlockPos, theme: &GenerationTheme) -> Generation {
        let theme = theme.clone();
        let s = Self {
            generation_type: theme.generation_types[0].clone(),
            theme,
            start: BlockPos::new(0, 0, 0),
        };

        let yaw = random_yaw();

        let mut g = s.generate(JumpDirection::DoesntMatter, yaw, Vec::new()); // no lines for first generation

        g.offset = start;
        g.end_state = PredictionState::running_jump_block(start, yaw);

        g
    }

    pub fn next_in_generation(
        direction: JumpDirection,
        theme: &GenerationTheme,
        generation: &Generation,
    ) -> Generation {
        let theme = theme.clone();
        let mut state = generation.end_state.clone();
        let mut lines = Vec::new();

        let target_y = (state.pos.y as i32 + direction.get_y_offset()) as f64;

        let g = loop {
            let mut new_state = state.clone();
            new_state.tick();

            if new_state.vel.y > 0. || new_state.pos.y > target_y {
                lines.push(Line3::new(state.pos.as_vec3(), new_state.pos.as_vec3()));
                state = new_state;
            } else {
                break Self {
                    generation_type: theme.get_random_generation_type(),
                    theme,
                    start: state.get_block_pos(),
                };
            }
        };

        g.generate(direction, generation.end_state.yaw, lines)
    }

    pub fn generate(
        &self,
        direction: JumpDirection,
        yaw: f32,
        mut lines: Vec<Line3>,
    ) -> Generation {
        let mut blocks = HashMap::new();
        let mut alt_blocks = HashMap::new();
        let mut offset: BlockPos = self.start;
        let mut children = Vec::new();
        let mut ordered = true;
        let end_state: PredictionState;

        let params = BlockGenParams {
            direction,
            block_map: self.theme.block_map.clone().build(),
        };

        match &self.generation_type {
            GenerationType::Single(key) => {
                blocks.insert(BlockPos::new(0, 0, 0), params.block_map.get_block(key));

                end_state = PredictionState::running_jump_block(self.start, random_yaw());
            }
            GenerationType::Ramp(key) => {
                let new_yaw = random_yaw();

                let height = ((yaw - new_yaw).abs()).round() as i32 + 1;
                let down = direction.go_down();

                let yaw_change = (new_yaw - yaw) / height as f32;

                let get_block = || params.block_map.get_block(key);

                let get_slab = || {
                    let slab = params.block_map.get_slab(key);
                    (slab, slab.set(PropName::Type, PropValue::Top))
                };

                let mut pos = Vec3::new(0., 0., 0.);

                let mut curr_yaw = yaw;

                let get_pos_left =
                    |pos: Vec3, yaw: f32| pos + (Vec3::new(yaw.cos(), 0., yaw.sin()) * 2f32.sqrt());

                let get_pos_right = |pos: Vec3, yaw: f32| {
                    pos + (Vec3::new(-yaw.cos(), 0., -yaw.sin()) * 2f32.sqrt())
                };

                let mut block_map = HashMap::new();

                for _ in 0..height {
                    let left = get_pos_left(pos, curr_yaw);
                    let right = get_pos_right(pos, curr_yaw);

                    for b in get_blocks_between(left, right) {
                        block_map.entry(b).or_insert(get_block());
                    }

                    pos.x -= (curr_yaw.sin() * 2f32.sqrt()).clamp(-1., 1.);
                    pos.z += (curr_yaw.cos() * 2f32.sqrt()).clamp(-1., 1.);
                    pos = pos.round();

                    if !down {
                        pos.y += 1.;
                    }

                    let left = get_pos_left(pos, curr_yaw);
                    let right = get_pos_right(pos, curr_yaw);

                    for b in get_blocks_between(left, right) {
                        let c = BlockPos::new(b.x, b.y - 1, b.z);
                        if !block_map.contains_key(&c) {
                            let (slab, top) = get_slab();
                            block_map.entry(c).or_insert(top);
                            block_map.entry(b).or_insert(slab);
                        }
                    }

                    if down {
                        pos.y -= 1.;
                    }

                    curr_yaw += yaw_change;

                    pos.x -= (curr_yaw.sin() * 2f32.sqrt()).clamp(-1., 1.);
                    pos.z += (curr_yaw.cos() * 2f32.sqrt()).clamp(-1., 1.);
                    pos = pos.round();
                }

                let left = get_pos_left(pos, curr_yaw);
                let right = get_pos_right(pos, curr_yaw);

                for b in get_blocks_between(left, right) {
                    block_map.entry(b).or_insert(get_block());
                }
                block_map.entry(pos.to_block_pos()).or_insert(get_block());

                for (pos, block) in block_map {
                    blocks.insert(pos, block);
                }

                end_state = PredictionState::running_jump_block(
                    self.start + pos.round().to_block_pos(),
                    new_yaw,
                );
            }
            GenerationType::Island {
                grass,
                dirt,
                stone,
                water,
                min_radius,
                max_radius,
                min_point_power,
                max_point_power,
            } => {
                let island = IslandGenerator {
                    grass: grass.to_owned(),
                    dirt: dirt.to_owned(),
                    stone: stone.to_owned(),
                    water: water.to_owned(),
                    min_radius: *min_radius,
                    max_radius: *max_radius,
                    min_point_power: *min_point_power,
                    max_point_power: *max_point_power,
                };

                let gen = island.generate(&params);

                offset = offset - gen.start;
                blocks = gen.blocks;
                alt_blocks = gen.alt_blocks;
                children = gen.children;
                end_state =
                    PredictionState::running_jump_block(offset + gen.end, random_yaw_dist(30.));

                for line in gen.lines {
                    lines.push(line + offset.to_vec3());
                }
            }
            GenerationType::Indoor {
                walls,
                floor,
                platforms,
            } => {
                let indoor = IndoorGenerator {
                    walls: walls.to_owned(),
                    floor: floor.clone(),
                    platforms: platforms.to_owned(),
                };

                let gen = indoor.generate(&params); // TODO: Streamline this.

                offset = offset - gen.start;
                blocks = gen.blocks;
                children = gen.children;
                end_state =
                    PredictionState::running_jump_block(offset + gen.end, random_yaw_dist(30.)); // walls can be in the way

                for line in gen.lines {
                    lines.push(line + offset.to_vec3());
                }
            }
            GenerationType::Cave(block_name) => {
                let cave = CaveGenerator {
                    block_name: block_name.to_owned(),
                };

                let gen = cave.generate(&params);

                offset = offset - gen.start;
                blocks = gen.blocks;
                children = gen.children;
                end_state =
                    PredictionState::running_jump_block(offset + gen.end, random_yaw_dist(30.));

                for line in gen.lines {
                    lines.push(line + offset.to_vec3());
                }
            }
            GenerationType::Snake(block_name) => {
                // TODO: Add more options
                let mut rng = rand::thread_rng();
                let mut snake = SnakeGenerator {
                    block_name: block_name.to_owned(),
                    snake_count: 1,
                    snake_length: 1,
                    delay: 5,
                    reverse: rng.gen(),
                    poses: Vec::new(),
                    end_pos: BlockPos::new(0, 0, 0),
                };

                while snake.poses.len() < 15 {
                    snake.create_looping_snake(BlockPos::new(-10, 0, 0), BlockPos::new(10, 0, 120));
                }

                let len = snake.poses.len();

                snake.snake_count = rng.gen_range(2..=4.min(len / 7));

                while len % snake.snake_count != 0 {
                    snake.snake_count = rng.gen_range(1..=4.min(len / 7));
                }

                snake.snake_length =
                    rng.gen_range(len / snake.snake_count / 2..=len / snake.snake_count * 3 / 4);
                let ratio =
                    snake.snake_length as f32 * snake.snake_count as f32 / snake.poses.len() as f32;

                if snake.snake_length > 100 {
                    // 1 is just way too fast
                    if ratio > 0.7 {
                        snake.delay = 2;
                    } else {
                        snake.delay = rng.gen_range(2..=3);
                    }
                } else if snake.snake_length > 55 {
                    snake.delay = rng.gen_range(2..=3);
                } else if snake.snake_length > 35 {
                    snake.delay = rng.gen_range(2..=4);
                } else if snake.snake_length > 25 {
                    snake.delay = rng.gen_range(3..=5);
                } else if snake.snake_length > 15 {
                    snake.delay = rng.gen_range(4..=6);
                } else {
                    snake.delay = rng.gen_range(5..=7);
                }

                let gen = snake.generate(&params);

                offset = offset - gen.start;
                blocks = gen.blocks;
                alt_blocks = gen.alt_blocks;
                children = gen.children;
                end_state =
                    PredictionState::running_jump_block(offset + gen.end, random_yaw_dist(30.));
                ordered = false;

                for line in gen.lines {
                    lines.push(line + offset.to_vec3());
                }
            }
            GenerationType::BlinkBlocks {
                on,
                off,
                delay,
                overlap,
            } => {
                // TODO: Add more options
                let blink_blocks = BlinkBlocksGenerator {
                    on: on.to_owned(),
                    off: off.to_owned(),
                    size: IVec2::new(3, 3),
                    delay: *delay,
                    overlap: *overlap,
                };

                let gen = blink_blocks.generate(&params);

                offset = offset - gen.start;
                blocks = gen.blocks;
                alt_blocks = gen.alt_blocks;
                children = gen.children;
                end_state =
                    PredictionState::running_jump_block(offset + gen.end, random_yaw_dist(30.));

                for line in gen.lines {
                    lines.push(line + offset.to_vec3());
                }
            }
            GenerationType::SingleCustom(preset) => {
                let gen = preset.generate(&params);

                offset = offset - gen.start;
                blocks = gen.blocks;
                children = gen.children;
                end_state =
                    PredictionState::running_jump_block(offset + gen.end, random_yaw_dist(30.));

                for line in gen.lines {
                    lines.push(line + offset.to_vec3());
                }
            }
            GenerationType::MultiCustom(preset) => {
                let gen = preset.generate(&params);

                offset = offset - gen.start;
                blocks = gen.blocks;
                children = gen.children;
                end_state =
                    PredictionState::running_jump_block(offset + gen.end, random_yaw_dist(30.));

                for line in gen.lines {
                    lines.push(line + offset.to_vec3());
                }
            }
        }

        Generation {
            blocks,
            children,
            alt_blocks,
            ordered,
            offset,
            end_state,
            lines,
        }
    }
}

/// The `BlockGenerator` trait represents a block generator.
pub trait BlockGenerator {
    /// The `generate` method generates blocks.
    fn generate(&self, params: &BlockGenParams) -> GenerateResult;
}

/// The `BlockGenParams` struct represents parameters for a block generator.
#[derive(Clone, Debug)]
pub struct BlockGenParams {
    pub direction: JumpDirection,
    pub block_map: BuiltBlockCollectionMap,
}
