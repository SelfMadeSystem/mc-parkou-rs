use std::collections::HashMap;

use crate::{alt_block::*, line::Line3, prediction::prediction_state::PredictionState, utils::*};

use super::{block_collection::*, generation::*, theme::GenerationTheme, generators::*};
use rand::Rng;
use valence::prelude::*;

pub type GenerateResult = (
    BlockPos,
    HashMap<BlockPos, BlockState>,
    BlockPos,
    Vec<Line3>,
    Vec<ChildGeneration>,
);

/// The `GenerationType` enum represents the different types of parkour generations
/// that can be used.
///
/// Variants:
/// * `Single`: The `Single` variant represents a single block.
/// * `Slime`: The `Slime` variant represents a slime block.
/// * `Ramp`: The `Ramp` variant represents blocks and slabs that are used to create
/// a ramp.
/// * `Island`: The `Island` variant represents blocks that are used to create an
/// island.
/// * `Bridge`: The `Bridge` variant represents blocks and slabs that are used to
/// create a bridge as well as wall blocks.
/// * `Indoor`: The `Indoor` variant represents blocks that are used to create an
/// indoor area.
/// * `Cave`: The `Cave` variant represents blocks that are used to create a cave.
/// * `Snake`: The `Snake` variant represents blocks that are used to create a
/// snake.
/// * `Custom`: The `Custom` variant represents a custom parkour generation. It has
/// preset blocks, a start position, and an end position.
#[derive(Clone, Debug)]
pub enum GenerationType {
    Single(BlockCollection),
    // Slime,
    Ramp(BlockSlabCollection),
    // Island(TerrainBlockCollection),
    // Bridge(BridgeBlockCollection),
    Indoor(IndoorBlockCollection),
    Cave(BlockCollection),
    Snake(BlockCollection),
    // Custom(CustomGeneration),
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
        let mut rng = rand::thread_rng();

        let target_y = match direction {
            JumpDirection::Up => state.pos.y as i32 + 1,
            JumpDirection::Down => state.pos.y as i32 - rng.gen_range(1..=2),
            JumpDirection::DoesntMatter => state.pos.y as i32 + rng.gen_range(-1..=1),
        } as f64;

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
                let mut rng = rand::thread_rng();
                let new_yaw = random_yaw();

                let height = ((yaw - new_yaw).abs()).round() as i32 + 1;
                let down = match direction {
                    JumpDirection::Up => true,
                    JumpDirection::Down => false,
                    JumpDirection::DoesntMatter => rng.gen(),
                };

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
                block_map.entry(pos.as_block_pos()).or_insert(get_block());

                for (pos, block) in block_map {
                    blocks.insert(pos, block);
                }

                end_state = PredictionState::running_jump_block(
                    self.start + pos.round().as_block_pos(),
                    new_yaw,
                );
            }
            GenerationType::Indoor(collection) => {
                let indoor = IndoorGenerator::new(collection.clone());

                let (start, bloccs, end, linez, childrenz) = indoor.generate();

                offset = offset - start;
                blocks = bloccs;
                children = childrenz;
                end_state = PredictionState::running_jump_block(offset + end, random_yaw_dist(30.)); // walls can be in the way

                for line in linez {
                    lines.push(line + offset.to_vec3());
                }
            }
            GenerationType::Cave(BlockCollection(collection)) => {
                let cave = CaveGenerator::new(collection.clone());

                let (start, bloccs, end, linez, childrenz) = cave.generate();

                offset = offset - start;
                blocks = bloccs;
                children = childrenz;
                end_state = PredictionState::running_jump_block(offset + end, random_yaw_dist(30.));

                for line in linez {
                    lines.push(line + offset.to_vec3());
                }
            }
            GenerationType::Snake(BlockCollection(collection)) => {
                // TODO: Make this better
                // For now, it's just large square
                let mut rng = rand::thread_rng();

                let size = rng.gen_range(4..=14);
                let snake_count = 2;
                let total_blocks = (size as u32 * 4 - 4) / snake_count;
                let snake_length = size as u32 * 4 / 5;
                let delay = rng.gen_range(4..10);
                let block = collection.blocks.get_random().unwrap().clone();

                for i in 0..size {
                    {
                        let x = i - size / 2;
                        let z = 0;

                        blocks.insert(
                            BlockPos::new(x, 0, z),
                            BlockState::BARRIER, // doesn't matter as it will be replaced
                        );

                        alt_blocks.insert(
                            BlockPos::new(x, 0, z),
                            AltBlock::Tick(
                                vec![
                                    (AltBlockState::Block(block), snake_length * delay),
                                    (
                                        AltBlockState::SmallBlock(block),
                                        (total_blocks - snake_length) * delay,
                                    ),
                                ],
                                i as u32 * delay,
                            ),
                        );
                    }

                    if i > 0 && i < size - 1 {
                        let mut x = size / 2;
                        if size % 2 == 0 {
                            x -= 1;
                        }
                        let z = i;

                        blocks.insert(
                            BlockPos::new(x, 0, z),
                            BlockState::BARRIER, // doesn't matter as it will be replaced
                        );

                        alt_blocks.insert(
                            BlockPos::new(x, 0, z),
                            AltBlock::Tick(
                                vec![
                                    (AltBlockState::Block(block), snake_length * delay),
                                    (
                                        AltBlockState::SmallBlock(block),
                                        (total_blocks - snake_length) * delay,
                                    ),
                                ],
                                (i + size - 1) as u32 * delay,
                            ),
                        );
                    }

                    {
                        let mut x = size / 2 - i;
                        if size % 2 == 0 {
                            x -= 1;
                        }
                        let z = size - 1;

                        blocks.insert(
                            BlockPos::new(x, 0, z),
                            BlockState::BARRIER, // doesn't matter as it will be replaced
                        );

                        alt_blocks.insert(
                            BlockPos::new(x, 0, z),
                            AltBlock::Tick(
                                vec![
                                    (AltBlockState::Block(block), snake_length * delay),
                                    (
                                        AltBlockState::SmallBlock(block),
                                        (total_blocks - snake_length) * delay,
                                    ),
                                ],
                                (i + size * 2 - 2) as u32 * delay,
                            ),
                        );
                    }

                    if i > 0 && i < size - 1 {
                        let x = -size / 2;
                        let z = size - 1 - i;

                        blocks.insert(
                            BlockPos::new(x, 0, z),
                            BlockState::BARRIER, // doesn't matter as it will be replaced
                        );

                        alt_blocks.insert(
                            BlockPos::new(x, 0, z),
                            AltBlock::Tick(
                                vec![
                                    (AltBlockState::Block(block), snake_length * delay),
                                    (
                                        AltBlockState::SmallBlock(block),
                                        (total_blocks - snake_length) * delay,
                                    ),
                                ],
                                (i + size * 3 - 3) as u32 * delay,
                            ),
                        );
                    }
                }

                end_state = PredictionState::running_jump_block(
                    BlockPos {
                        x: self.start.x,
                        y: self.start.y,
                        z: self.start.z + size - 1,
                    },
                    random_yaw(),
                );
            }
        }

        Generation::new(blocks, children, alt_blocks, offset, end_state, lines)
    }
}
