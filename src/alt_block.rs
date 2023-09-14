use std::collections::HashMap;

use valence::{layer::chunk::IntoBlock, prelude::*, entity::{block_display::{BlockDisplayEntityBundle, self}, display}};

use crate::utils::*;

/// An `AltBlock` is a block that changes under certain conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum AltBlock {
    /// A block that changes between different `AltBlockState`s every certain amount of ticks.
    /// The first parameter is a vector of tuples. The first element of the tuple is the `AltBlockState`
    /// to change to. The second element of the tuple is the amount of ticks to wait before changing
    /// to the next `AltBlockState`. The second parameter is the offset of the ticks.
    Tick(Vec<(AltBlockState, u32)>, u32),
    // /// A block that changes between alternating `AltBlockState`s every time the player jumps.
    // Jump(Vec<AltBlockState>), // TODO: Implement this.
    // /// A block that changes between alternating `AltBlockState`s every time it is stepped on.
    // Step(Vec<AltBlockState>), // TODO: Implement this.
}

impl AltBlock {
    /// Returns an `AltBlockState` of the `AltBlock` with the given parameters.
    ///
    /// # Parameters
    ///
    /// * `params`: The `params` parameter is of type `AltBlockParams`. It represents the parameters
    /// of the current tick.
    pub fn get_block(&self, params: &AltBlockParams) -> AltBlockState {
        match self {
            AltBlock::Tick(blocks, offset) => {
                let mut total = 0;

                for (_, ticks) in blocks {
                    total += ticks;
                }

                let mut tick = params.ticks + offset;

                tick %= total;

                for (block, ticks) in blocks {
                    if tick < *ticks {
                        return *block;
                    }

                    tick -= ticks;
                }

                blocks[0].0
            }
        }
    }
}

/// The `AltBlockParams` struct represents the parameters of the current tick.
///
/// Properties:
///
/// * `ticks`: The `ticks` property is of type `u32`. It represents the amount of ticks that have
/// passed.
#[derive(Debug, Clone, PartialEq)]
pub struct AltBlockParams {
    pub ticks: u32,
    // TODO: Add more parameters. (e.g. player position, player velocity, block position, etc.)
}

/// An `AltBlockState` is a state of an `AltBlock`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AltBlockState {
    /// A regular block.
    Block(BlockState),
    /// A smaller, non-solid block. I.e., a `BlockDisplay` with a smaller size.
    SmallBlock(BlockState),
}

impl AltBlockState {
    /// Sets the block in the world.
    pub fn set_block(
        &self,
        pos: BlockPos,
        alt_block_entities: &mut HashMap<BlockPos, Entity>,
        prev_alt_block_states: &mut HashMap<BlockPos, AltBlockState>,
        commands: &mut Commands,
        world: &mut ChunkLayer,
        layer: &EntityLayerId,
    ) {
        if prev_alt_block_states.contains_key(&pos) && *self == prev_alt_block_states[&pos] {
            return;
        }

        prev_alt_block_states.insert(pos, *self);

        match self {
            AltBlockState::Block(block) => {
                if alt_block_entities.contains_key(&pos) {
                    if let Some(mut entity) = commands.get_entity(alt_block_entities[&pos]) {
                        entity.insert(Despawned);
                    }
                    alt_block_entities.remove(&pos);
                }
                world.set_block(pos, block.into_block());
            }
            AltBlockState::SmallBlock(block) => {
                if alt_block_entities.contains_key(&pos) {
                    if let Some(mut entity) = commands.get_entity(alt_block_entities[&pos]) {
                        entity.insert(Despawned);
                    }
                }

                world.set_block(pos, BlockState::AIR.into_block());

                let display = BlockDisplayEntityBundle {
                    position: Position(pos.to_vec3().as_dvec3()),
                    layer: *layer,
                    block_display_block_state: block_display::BlockState(*block),
                    display_scale: display::Scale(Vec3::new(0.5, 0.5, 0.5)),
                    display_translation: display::Translation(Vec3::new(0.25, 0.25, 0.25)),
                    ..Default::default()
                };
                
                let entity = commands.spawn(display).id();

                alt_block_entities.insert(pos, entity);
            }
        }
    }
}
