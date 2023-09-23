use std::collections::HashMap;

use crate::{alt_block::*, line::Line3, prediction::prediction_state::PredictionState, utils::*};

use super::{block_collection::*, generation::*, generators::*, theme::GenerationTheme, custom_generation::{SingleCustomPreset, MultiCustomPreset}};
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
    pub fn just_blocks(blocks: HashMap<BlockPos, BlockState>, start: BlockPos, end: BlockPos) -> Self {
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
/// // TODO
/// * `MultiCustom`: The `MultiCustom` variant represents a custom parkour
/// generation. It has a start custom generation, a number of middle custom
/// generations, and an end custom generation. // TODO
#[derive(Clone, Debug)]
pub enum GenerationType {
    Single(BlockCollection),
    // Slime,
    Ramp(BlockSlabCollection),
    // Island(TerrainBlockCollection),
    Indoor(IndoorBlockCollection),
    Cave(BlockCollection),
    Snake(BlockCollection),
    BlinkBlocks(BlinkBlockCollection),
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

        match &self.generation_type {
            GenerationType::Single(BlockCollection(collection)) => {
                blocks.insert(
                    BlockPos::new(0, 0, 0),
                    *collection
                        .blocks
                        .get_random()
                        .expect("No blocks in block collection"),
                );

                end_state = PredictionState::running_jump_block(self.start, random_yaw());
            }
            GenerationType::Ramp(BlockSlabCollection(collection)) => {
                // TODO: Not great. Should be a better way to do this.
                let index = collection.blocks.get_random_index().unwrap();
                let uniform = collection.uniform;
                let new_yaw = random_yaw();

                let height = ((yaw - new_yaw).abs()).round() as i32 + 1;
                let down = direction.go_down();

                let yaw_change = (new_yaw - yaw) / height as f32;

                let get_block_slab = || {
                    if uniform {
                        collection.blocks[index].clone()
                    } else {
                        collection.blocks.get_random().unwrap().clone()
                    }
                };

                let get_block = || get_block_slab().block;

                let get_slab = || {
                    let slab = get_block_slab().slab;
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
            GenerationType::Indoor(collection) => {
                let indoor = IndoorGenerator::new(collection.clone());

                let gen = indoor.generate(); // TODO: Streamline this.

                offset = offset - gen.start;
                blocks = gen.blocks;
                children = gen.children;
                end_state =
                    PredictionState::running_jump_block(offset + gen.end, random_yaw_dist(30.)); // walls can be in the way

                for line in gen.lines {
                    lines.push(line + offset.to_vec3());
                }
            }
            GenerationType::Cave(BlockCollection(collection)) => {
                let cave = CaveGenerator::new(collection.clone());

                let gen = cave.generate();

                offset = offset - gen.start;
                blocks = gen.blocks;
                children = gen.children;
                end_state =
                    PredictionState::running_jump_block(offset + gen.end, random_yaw_dist(30.));

                for line in gen.lines {
                    lines.push(line + offset.to_vec3());
                }
            }
            GenerationType::Snake(BlockCollection(collection)) => {
                let mut rng = rand::thread_rng();
                let mut snake = SnakeGenerator::new(collection.clone(), 1, 1, 5, rng.gen());

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

                let gen = snake.generate();

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
            GenerationType::BlinkBlocks(collection) => {
                let blink_blocks = BlinkBlocksGenerator::new(collection.clone(), IVec2::new(3, 3));

                let gen = blink_blocks.generate(direction);

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
                let gen = preset.generate();

                offset = offset - gen.start;
                blocks = gen.blocks;
                children = gen.children;
                end_state =
                    PredictionState::running_jump_block(offset + gen.end, random_yaw_dist(30.));

                for line in gen.lines {
                    lines.push(line + offset.to_vec3());
                }
            },
            GenerationType::MultiCustom(preset) => {
                let gen = preset.generate();

                offset = offset - gen.start;
                blocks = gen.blocks;
                children = gen.children;
                end_state =
                    PredictionState::running_jump_block(offset + gen.end, random_yaw_dist(30.));

                for line in gen.lines {
                    lines.push(line + offset.to_vec3());
                }
            },
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
    /// TODO: Add parameters, such as jump direction, also want a block map to
    /// be passed
    fn generate(&self) -> GenerateResult;
}
