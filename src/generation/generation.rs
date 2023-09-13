use std::collections::HashMap;

use valence::{layer::chunk::IntoBlock, prelude::*};

use crate::{line::Line3, prediction::prediction_state::PredictionState};

/// The `Generation` struct represents a parkour generation.
///
/// Properties:
///
/// * `blocks`: The `blocks` property is of type `HashMap<BlockPos, Block>`. It represents
/// blocks that are generated.
/// * `offset`: The `offset` property is of type `BlockPos`. It represents the offset
/// of the parkour generation.
/// * `end_state`: The `end_state` property is of type `PredictionState`. It represents
/// the state to expect the player to be in at the end of the parkour generation.
/// * `lines`: The `lines` property is of type `Vec<Line3>`. It represents the path the
/// player takes through the parkour generation.
#[derive(Clone, Debug)]
pub struct Generation {
    pub blocks: HashMap<BlockPos, Block>,
    pub offset: BlockPos,
    pub end_state: PredictionState,
    pub lines: Vec<Line3>,
}

impl Generation {
    pub fn new(
        blocks: HashMap<BlockPos, Block>,
        offset: BlockPos,
        end_state: PredictionState,
        lines: Vec<Line3>,
    ) -> Self {
        Self {
            blocks,
            offset,
            end_state,
            lines,
        }
    }

    pub fn place(&self, world: &mut ChunkLayer) {
        for (pos, block) in &self.blocks {
            world.set_block(*pos + self.offset, block.clone());
        }
    }

    pub fn remove(&self, world: &mut ChunkLayer) {
        for (pos, _) in &self.blocks {
            world.set_block(*pos + self.offset, BlockState::AIR.into_block());
        }
    }

    /// Returns true if the player has reached any of the blocks.
    pub fn has_reached(&self, pos: Position) -> bool {
        let pos = BlockPos::new(
            (pos.0.x - 0.5).round() as i32,
            pos.0.y as i32 - 1,
            (pos.0.z - 0.5).round() as i32,
        ) - self.offset;

        self.blocks.contains_key(&pos) || self.blocks.contains_key(&(pos + BlockPos::new(0, 1, 0)))
    }
}
