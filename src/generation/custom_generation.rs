use std::collections::HashMap;

use valence::prelude::*;

use super::{block_collection::BlockCollectionMap, generator::{BlockGenerator, GenerateResult}};

/// The `SingleCustomPreset` struct represents a single custom generation preset.
/// It is used to store the blocks used in a custom generation preset.
///
/// Properties:
///
/// * `block_map`: The `block_map` property is a `BlockCollectionMap`. It maps a
/// name to a `BlockCollection`.
/// * `blocks`: The `blocks` property is a `HashMap<BlockPos, String>`. It maps a
/// position to a block name.
/// * `start_pos`: The `start_pos` property is a `BlockPos`. It represents the
/// starting position of the custom generation preset.
/// * `end_pos`: The `end_pos` property is a `BlockPos`. It represents the ending
/// position of the custom generation preset.
#[derive(Clone, Debug)]
pub struct SingleCustomPreset {
    pub block_map: BlockCollectionMap,
    pub blocks: HashMap<BlockPos, String>,
    pub start_pos: BlockPos,
    pub end_pos: BlockPos,
}

impl BlockGenerator for SingleCustomPreset {
    fn generate(&self) -> GenerateResult {
        let built = self.block_map.clone().build();
        let mut blocks = HashMap::new();
        for (pos, block_name) in self.blocks.iter() {
            let block = built.get_block(block_name).expect("Block not found");
            blocks.insert(*pos, block);
        }
        GenerateResult::just_blocks(
            blocks,
            self.start_pos,
            self.end_pos,
        )
    }
}
