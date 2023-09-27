use std::collections::HashMap;

use rand::{seq::SliceRandom, Rng};
use valence::prelude::*;

use crate::{
    generation::{
        block_collection::BuiltBlockCollectionMap,
        generation::ChildGeneration,
        generator::*,
    },
    line::Line3,
    prediction::prediction_state::PredictionState,
    utils::*,
    weighted_vec::WeightedVec,
};

type BlockProperties = HashMap<BlockPos, (String, Vec<(PropName, PropValue)>)>;

/// The `SingleCustomPreset` struct represents a single custom generation preset.
/// It is used to store the blocks used in a custom generation preset.
///
/// Properties:
///
/// * `blocks`: The `blocks` property is a `BlockProperties`. It maps a position
/// to a block name and a list of properties.
/// * `start_pos`: The `start_pos` property is a `BlockPos`. It represents the
/// starting position of the custom generation preset.
/// * `end_pos`: The `end_pos` property is a `BlockPos`. It represents the ending
/// position of the custom generation preset.
#[derive(Clone, Debug)]
pub struct SingleCustomPreset {
    pub blocks: BlockProperties,
    pub start_pos: BlockPos,
    pub end_pos: BlockPos,
}

impl SingleCustomPreset {
    fn get_blocks(
        &self,
        offset: BlockPos,
        map: &BuiltBlockCollectionMap,
    ) -> HashMap<BlockPos, BlockState> {
        let mut blocks = HashMap::new();
        for (pos, (block_name, props)) in self.blocks.iter() {
            let mut block = map.get_block(block_name);
            for (name, value) in props {
                block = block.set(*name, *value);
            }
            blocks.insert(*pos + offset, block);
        }

        blocks
    }

    fn generate_child(&self, offset: BlockPos, map: &BuiltBlockCollectionMap) -> ChildGeneration {
        ChildGeneration::new(self.get_blocks(offset, map), Default::default())
    }
}

impl BlockGenerator for SingleCustomPreset {
    fn generate(&self, params: &BlockGenParams) -> GenerateResult {
        GenerateResult::just_blocks(
            self.get_blocks(BlockPos::new(0, 0, 0), &params.block_map),
            self.start_pos,
            self.end_pos,
        )
    }
}

#[derive(Clone, Debug)]
pub struct SingularMultiCustomPreset {
    pub preset: SingleCustomPreset,
    pub nexts: Vec<String>,
    pub fixed_offset: Option<BlockPos>,
}

#[derive(Clone, Debug)]
pub struct MultiCustomPreset {
    pub presets: HashMap<String, SingularMultiCustomPreset>,
    pub start: WeightedVec<String>,
    pub end: WeightedVec<String>,
    pub min_length: i32,
    pub max_length: i32,
}

impl BlockGenerator for MultiCustomPreset {
    fn generate(&self, params: &BlockGenParams) -> GenerateResult {
        let mut rng = rand::thread_rng();
        let mut length = rng.gen_range(self.min_length..=self.max_length);
        let mut children = Vec::new();
        let mut lines = Vec::new();
        let mut current_pos = BlockPos::new(0, 0, 0);

        let mut start = true;
        let mut current = self.start.get_random().expect("No start");

        while length >= 0 {
            let preset = if length == 0 {
                self.presets
                    .get(current)
                    .expect(format!("No preset {}", current).as_str())
            } else {
                self.presets
                    .get(current)
                    .expect(format!("No preset {}", current).as_str())
            };
            let mut offset = if start {
                let a = preset.preset.start_pos;
                Some(BlockPos::new(-a.x, -a.y, -a.z))
            } else {
                preset.fixed_offset
            };

            if offset.is_none() {
                let mut prediction =
                    PredictionState::running_jump_block(BlockPos::new(0, 0, 0), random_yaw());
                let mut prev_pos = prediction.pos;

                let target_y = rand::thread_rng().gen_range(-1..=1) as f64;

                loop {
                    let mut new_prediction = prediction.clone();
                    new_prediction.tick();

                    if new_prediction.vel.y < 0.0 && new_prediction.pos.y <= target_y {
                        break;
                    } else {
                        lines.push(Line3::new(
                            prev_pos.as_vec3() + current_pos.to_vec3(),
                            new_prediction.pos.as_vec3() + current_pos.to_vec3(),
                        ));
                        prev_pos = new_prediction.pos;
                        prediction = new_prediction;
                    }
                }

                offset = Some(prediction.get_block_pos() - current_pos);
            }

            current_pos = offset.expect("This should never happen!") + current_pos;

            let mut child = preset.preset.generate_child(current_pos, &params.block_map);

            if start {
                child.reached = true; // TODO: I feel like there is a better way to do this
            }

            children.push(child);

            current_pos = current_pos + preset.preset.end_pos;

            start = false;
            length -= 1;

            if length == 0 {
                current = self.end.get_random().expect("No end");
            } else {
                current = preset.nexts.choose(&mut rng).expect("No next");
            }
        }

        GenerateResult {
            start: BlockPos::new(0, 0, 0),
            end: current_pos,
            blocks: HashMap::new(),
            alt_blocks: HashMap::new(),
            lines,
            children,
        }
    }
}
