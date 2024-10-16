use std::{collections::{HashMap, HashSet}, f32::consts::PI};

use crate::{alt_block::*, line::Line3, prediction::prediction_state::PredictionState, utils::*};

use super::{block_collection::*, generation::*, generators::*, theme::GenerationTheme};
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
/// * `Single`: The `Single` variant represents a single block..
#[derive(Clone, Debug)]
pub enum GenerationType {
    Single(String),
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

        let mut g = s.generate(JumpDirection::DoesntMatter, HashSet::new(), yaw, Vec::new(), &Vec::new()); // no lines for first generation

        g.offset = start;
        g.end_block = start;

        g
    }

    pub fn next_in_generation(
        direction: JumpDirection,
        theme: &GenerationTheme,
        generation: &Generation,
        prev_generations: &Vec<&Generation>,
    ) -> Generation {
        let blocks: Vec<_> = prev_generations
            .iter()
            .flat_map(|g| g.blocks.iter().map(|(pos, _)| *pos + g.offset.as_ivec3()))
            .collect();
        let jump_blocks: HashSet<_> = prev_generations
            .iter()
            .flat_map(|g| &g.jump_blocks)
            .collect();
        let theme = theme.clone();
        let mut lines = Vec::new();

        let start_block = generation.end_block;
        let target_y = (start_block.y + 1 + direction.get_y_offset()) as f64;

        let (g, yaw, jumped_blocks) = 'outer: loop {
            lines.clear();
            let yaw = random_yaw();
            let mut state = PredictionState::running_jump_block(start_block, yaw);
            let mut jumped_blocks = HashSet::new();
            loop {
                let mut new_state = state.clone();
                new_state.tick();

                if new_state.is_intersecting_any_block(&blocks) {
                    break;
                }

                if new_state.pos.y.floor() >= state.pos.y.floor() || new_state.pos.y > target_y {
                    lines.push(Line3::new(state.pos.as_vec3(), new_state.pos.as_vec3()));
                    jumped_blocks.extend(state.get_intersected_blocks());
                    state = new_state;
                } else {
                    let end_block = BlockPos::new(
                        state.pos.x.floor() as i32,
                        state.pos.y.floor() as i32,
                        state.pos.z.floor() as i32,
                    );
                    if jump_blocks.contains(&end_block) {
                        break;
                    }
                    if blocks.iter().take(blocks.len() - 1).any(|b| {
                        (b.x == end_block.x
                            && b.z == end_block.z
                            && (b.y - end_block.y).abs() <= 2)
                            || prediction_can_reach(*b, end_block)
                    }) {
                        break;
                    }
                    jumped_blocks.extend(state.get_intersected_blocks());
                    break 'outer (
                        Self {
                            generation_type: theme.get_random_generation_type(),
                            theme,
                            start: state.get_block_pos(),
                        },
                        yaw,
                        jumped_blocks,
                    );
                }
            }
        };

        g.generate(direction, jumped_blocks, yaw, lines, prev_generations)
    }

    pub fn generate(
        &self,
        direction: JumpDirection,
        jump_blocks: HashSet<BlockPos>,
        yaw: f32,
        mut lines: Vec<Line3>,
        prev_generations: &Vec<&Generation>,
    ) -> Generation {
        let mut blocks = HashMap::new();
        let mut alt_blocks = HashMap::new();
        let mut offset: BlockPos = self.start;
        let mut children = Vec::new();
        let mut ordered = true;
        let end_block;

        let params = BlockGenParams {
            direction,
            block_map: self.theme.block_map.clone().build(),
        };

        match &self.generation_type {
            GenerationType::Single(key) => {
                blocks.insert(BlockPos::new(0, 0, 0), params.block_map.get_block(key));

                // end_state = PredictionState::running_jump_block(self.start, random_yaw());
                end_block = self.start;
            }
        }

        Generation {
            blocks,
            jump_blocks,
            children,
            alt_blocks,
            ordered,
            offset,
            end_block,
            lines,
        }
    }
}

/// The `BlockGenerator` trait represents a block generator.
pub trait BlockGenerator {
    /// The `generate` method generates blocks.
    /// TODO: Make &self mutable
    fn generate(&self, params: &BlockGenParams) -> GenerateResult;
}

/// The `BlockGenParams` struct represents parameters for a block generator.
#[derive(Clone, Debug)]
pub struct BlockGenParams {
    pub direction: JumpDirection,
    pub block_map: BuiltBlockCollectionMap,
}
