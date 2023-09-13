use std::collections::HashMap;

use valence::{layer::chunk::IntoBlock, prelude::*};

use crate::{line::Line3, prediction::prediction_state::PredictionState, utils::*};

/// The `Generation` struct represents a parkour generation.
///
/// Properties:
///
/// * `blocks`: The `blocks` property is of type `HashMap<BlockPos, Block>`. It represents
/// blocks that are generated.
/// * `children`: The `children` property is of type `Vec<ChildGeneration>`. It represents
/// child generations that are generated.
/// * `offset`: The `offset` property is of type `BlockPos`. It represents the offset
/// of the parkour generation.
/// * `end_state`: The `end_state` property is of type `PredictionState`. It represents
/// the state to expect the player to be in at the end of the parkour generation.
/// * `lines`: The `lines` property is of type `Vec<Line3>`. It represents the path the
/// player takes through the parkour generation.
#[derive(Clone, Debug)]
pub struct Generation {
    pub blocks: HashMap<BlockPos, Block>,
    pub children: Vec<ChildGeneration>,
    pub offset: BlockPos,
    pub end_state: PredictionState,
    pub lines: Vec<Line3>,
}

impl Generation {
    pub fn new(
        blocks: HashMap<BlockPos, Block>,
        children: Vec<ChildGeneration>,
        offset: BlockPos,
        end_state: PredictionState,
        lines: Vec<Line3>,
    ) -> Self {
        Self {
            blocks,
            children,
            offset,
            end_state,
            lines,
        }
    }

    /// Places the blocks in the generation.
    pub fn place(&self, world: &mut ChunkLayer) {
        for (pos, block) in &self.blocks {
            world.set_block(*pos + self.offset, block.clone());
        }

        for child in &self.children {
            child.place(world, self.offset);
        }
    }

    /// Removes the blocks in the generation.
    pub fn remove(&self, world: &mut ChunkLayer) {
        for (pos, _) in &self.blocks {
            world.set_block(*pos + self.offset, BlockState::AIR.into_block());
        }

        for child in &self.children {
            child.remove(world, self.offset);
        }
    }

    /// Returns true if the player has reached any of the blocks.
    pub fn has_reached(&self, pos: Position) -> bool {
        let poses = get_player_floor_blocks(pos.0 - self.offset.to_vec3().as_dvec3());

        for pos in poses {
            if self.blocks.contains_key(&(pos)) {
                return true;
            }

            for child in &self.children {
                if child.blocks.contains_key(&(pos)) {
                    return true;
                }
            }
        }

        false
    }

    /// Returns true if a child generation has been reached (if it hasn't been reached yet).
    /// If a child generation has been reached, it will be marked as reached.
    pub fn has_reached_child(&mut self, pos: Position) -> bool {
        for child in &mut self.children {
            if child.has_reached(pos, self.offset) {
                return true;
            }
        }

        false
    }
}

/// The `ChildGeneration` struct represents a child generation.
///
/// Properties:
///
/// * `blocks`: The `blocks` property is of type `HashMap<BlockPos, Block>`. It represents
/// blocks that are generated.
/// * `reached`: The `reached` property is of type `bool`. It represents whether or not
/// the child generation has been reached by the player.
#[derive(Clone, Debug)]
pub struct ChildGeneration {
    pub blocks: HashMap<BlockPos, Block>,
    pub reached: bool,
}

impl ChildGeneration {
    pub fn new(blocks: HashMap<BlockPos, Block>) -> Self {
        Self {
            blocks,
            reached: false,
        }
    }

    /// Places the blocks in the generation.
    pub fn place(&self, world: &mut ChunkLayer, offset: BlockPos) {
        for (pos, block) in &self.blocks {
            world.set_block(*pos + offset, block.clone());
        }
    }

    /// Removes the blocks in the generation.
    pub fn remove(&self, world: &mut ChunkLayer, offset: BlockPos) {
        for (pos, _) in &self.blocks {
            world.set_block(*pos + offset, BlockState::AIR.into_block());
        }
    }

    /// Returns true if the player has reached any of the blocks.
    /// If so, the child generation will be marked as reached.
    pub fn has_reached(&mut self, pos: Position, offset: BlockPos) -> bool {
        if self.reached {
            return false;
        }

        let poses = get_player_floor_blocks(pos.0 - offset.to_vec3().as_dvec3());

        for pos in poses {
            if self.blocks.contains_key(&(pos)) {
                self.reached = true;
                return true;
            }
        }

        false
    }
}
