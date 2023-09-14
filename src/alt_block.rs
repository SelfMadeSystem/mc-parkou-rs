use valence::prelude::*;

/// An `AltBlock` is a block that changes under certain conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum AltBlock {
    /// A block that changes between different `BlockState`s every certain amount of ticks.
    /// The first parameter is a vector of tuples. The first element of the tuple is the `BlockState`
    /// to change to. The second element of the tuple is the amount of ticks to wait before changing
    /// to the next `BlockState`. The second parameter is the offset of the ticks.
    Tick(Vec<(BlockState, u32)>, u32),
    // /// A block that changes between alternating `BlockState`s every time the player jumps.
    // Jump(Vec<BlockState>), // TODO: Implement this.
    // /// A block that changes between alternating `BlockState`s every time it is stepped on.
    // Step(Vec<BlockState>), // TODO: Implement this.
}

impl AltBlock {
    /// Returns the `BlockState` of the `AltBlock` with the given parameters.
    /// 
    /// # Parameters
    /// 
    /// * `params`: The `params` parameter is of type `AltBlockParams`. It represents the parameters
    /// of the current tick.
    pub fn get_block(&self, params: &AltBlockParams) -> BlockState {
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
