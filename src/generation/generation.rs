use std::collections::HashMap;

use valence::{layer::chunk::IntoBlock, prelude::*};

use crate::{alt_block::*, line::Line3, prediction::prediction_state::PredictionState, utils::*};

/// The `Generation` struct represents a parkour generation.
///
/// Properties:
///
/// * `blocks`: The `blocks` property is of type `HashMap<BlockPos, BlockState>`. It represents
/// blocks that are generated.
/// * `children`: The `children` property is of type `Vec<ChildGeneration>`. It represents
/// child generations that are generated.
/// * `alt_blocks`: The `alt_blocks` property is of type `HashMap<BlockPos, AltBlock>`. It
/// represents blocks that change under certain conditions.
/// * `offset`: The `offset` property is of type `BlockPos`. It represents the offset
/// of the parkour generation.
/// * `end_state`: The `end_state` property is of type `PredictionState`. It represents
/// the state to expect the player to be in at the end of the parkour generation.
/// * `lines`: The `lines` property is of type `Vec<Line3>`. It represents the path the
/// player takes through the parkour generation.
#[derive(Clone, Debug)]
pub struct Generation {
    pub blocks: HashMap<BlockPos, BlockState>,
    pub children: Vec<ChildGeneration>,
    pub alt_blocks: HashMap<BlockPos, AltBlock>,
    pub offset: BlockPos,
    pub end_state: PredictionState,
    pub lines: Vec<Line3>,
}

impl Generation {
    pub fn new(
        blocks: HashMap<BlockPos, BlockState>,
        children: Vec<ChildGeneration>,
        alt_blocks: HashMap<BlockPos, AltBlock>,
        offset: BlockPos,
        end_state: PredictionState,
        lines: Vec<Line3>,
    ) -> Self {
        Self {
            blocks,
            children,
            alt_blocks,
            offset,
            end_state,
            lines,
        }
    }

    /// Places the blocks in the generation.
    pub fn place(&self, world: &mut ChunkLayer) {
        for (pos, block) in &self.blocks {
            world.set_block(*pos + self.offset, *block);
        }

        for child in &self.children {
            child.place(world, self.offset);
        }
    }

    /// Removes the blocks in the generation.
    ///
    /// TODO: Remove the entities in the alt blocks.
    pub fn remove(
        &self,
        world: &mut ChunkLayer,
        alt_block_entities: &mut HashMap<BlockPos, Entity>,
        prev_alt_block_states: &mut HashMap<BlockPos, AltBlockState>,
        commands: &mut Commands,
    ) {
        for (pos, _) in &self.blocks {
            world.set_block(*pos + self.offset, BlockState::AIR.into_block());
        }

        for (pos, _) in &self.alt_blocks {
            let pos = *pos + self.offset;
            if let Some(entity) = alt_block_entities.get_mut(&pos) {
                if let Some(mut entity) = commands.get_entity(*entity) {
                    entity.insert(Despawned);
                }

                alt_block_entities.remove(&pos);
                prev_alt_block_states.remove(&pos);
            }
        }

        for child in &self.children {
            child.remove(
                world,
                alt_block_entities,
                prev_alt_block_states,
                commands,
                self.offset,
            );
        }
    }

    /// Updates the alt blocks in the generation.
    pub fn update_alt_blocks(
        &self,
        params: &AltBlockParams,
        alt_block_entities: &mut HashMap<BlockPos, Entity>,
        prev_alt_block_states: &mut HashMap<BlockPos, AltBlockState>,
        commands: &mut Commands,
        world: &mut ChunkLayer,
        layer: &EntityLayerId,
    ) {
        for (pos, block) in &self.alt_blocks {
            let block = block.get_block(params);
            block.set_block(
                *pos + self.offset,
                alt_block_entities,
                prev_alt_block_states,
                commands,
                world,
                layer,
            )
        }

        for child in &self.children {
            child.update_alt_blocks(
                params,
                alt_block_entities,
                prev_alt_block_states,
                commands,
                world,
                layer,
                self.offset,
            );
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

    /// Returns the number to increment the score by from the child generations.
    pub fn has_reached_child(&mut self, pos: Position) -> u32 {
        let mut reached_count = 0;
        for (i, child) in &mut self.children.iter_mut().enumerate() {
            let i = i as u32;
            if child.reached {
                reached_count += 1;
                continue;
            }
            if child.has_reached(pos, self.offset) {
                if reached_count < i {
                    for i in 0..i as usize {
                        self.children[i].reached = true;
                    }
                }
                return i - reached_count + 1;
            }
        }

        0
    }

    /// Returns the number of child generations that have been not been reached.
    pub fn get_unreached_child_count(&self) -> u32 {
        let mut count = 0;
        for child in &self.children {
            if !child.reached {
                count += 1;
            }
        }

        count
    }
}

/// The `ChildGeneration` struct represents a child generation.
///
/// Properties:
///
/// * `blocks`: The `blocks` property is of type `HashMap<BlockPos, &BlockState>`. It represents
/// blocks that are generated.
/// * `alt_blocks`: The `alt_blocks` property is of type `HashMap<BlockPos, AltBlock>`. It
/// represents blocks that change under certain conditions.
/// * `reached`: The `reached` property is of type `bool`. It represents whether or not
/// the child generation has been reached by the player.
#[derive(Clone, Debug)]
pub struct ChildGeneration {
    pub blocks: HashMap<BlockPos, BlockState>,
    pub alt_blocks: HashMap<BlockPos, AltBlock>,
    pub reached: bool,
}

impl ChildGeneration {
    pub fn new(
        blocks: HashMap<BlockPos, BlockState>,
        alt_blocks: HashMap<BlockPos, AltBlock>,
    ) -> Self {
        Self {
            blocks,
            alt_blocks,
            reached: false,
        }
    }

    /// Places the blocks in the generation.
    pub fn place(&self, world: &mut ChunkLayer, offset: BlockPos) {
        for (pos, block) in &self.blocks {
            world.set_block(*pos + offset, *block);
        }
    }

    /// Removes the blocks in the generation.
    pub fn remove(
        &self,
        world: &mut ChunkLayer,
        alt_block_entities: &mut HashMap<BlockPos, Entity>,
        prev_alt_block_states: &mut HashMap<BlockPos, AltBlockState>,
        commands: &mut Commands,
        offset: BlockPos,
    ) {
        for (pos, _) in &self.blocks {
            world.set_block(*pos + offset, BlockState::AIR.into_block());
        }

        for (pos, _) in &self.alt_blocks {
            let pos = *pos + offset;
            if let Some(entity) = alt_block_entities.get_mut(&pos) {
                if let Some(mut entity) = commands.get_entity(*entity) {
                    entity.insert(Despawned);
                }

                alt_block_entities.remove(&pos);
                prev_alt_block_states.remove(&pos);
            }
        }
    }

    /// Updates the alt blocks in the generation.
    pub fn update_alt_blocks(
        &self,
        params: &AltBlockParams,
        alt_block_entities: &mut HashMap<BlockPos, Entity>,
        prev_alt_block_states: &mut HashMap<BlockPos, AltBlockState>,
        commands: &mut Commands,
        world: &mut ChunkLayer,
        layer: &EntityLayerId,
        offset: BlockPos,
    ) {
        for (pos, block) in &self.alt_blocks {
            let block = block.get_block(params);
            block.set_block(
                *pos + offset,
                alt_block_entities,
                prev_alt_block_states,
                commands,
                world,
                layer,
            )
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
