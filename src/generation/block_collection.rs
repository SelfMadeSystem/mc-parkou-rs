use std::collections::HashMap;

use crate::weighted_vec::WeightedVec;
use valence::prelude::*;

/// The `BlockChoice` struct represents a choice between blocks of type `T`, with
/// the option to choose only one block for a specific generation or to choose
/// multiple blocks with a weighted probability.
/// 
/// Properties:
/// 
/// * `blocks`: The `blocks` property is a `WeightedVec<T>`, which is a vector of
/// elements of type `T` with associated weights. Each element in the vector is
/// assigned a weight, which determines the probability of that element being
/// chosen.
/// * `uniform`: The `uniform` property is a boolean value that determines whether
/// the `BlockChoice` will choose only one block or multiple blocks. If `uniform`
/// is `true`, then only one block will be chosen. If `uniform` is `false`, then
/// it will choose a random block each time.
#[derive(Clone, Debug)]
pub struct BlockChoice<T> {
    pub blocks: WeightedVec<T>,
    pub uniform: bool,
}

#[derive(Clone, Debug)]
pub struct BlockCollection(pub BlockChoice<BlockState>);

#[derive(Clone, Debug)]
pub struct BlockSlabCollection(pub BlockChoice<BlockSlab>);

#[derive(Clone, Debug)]
pub struct BlockSlab {
    pub block: BlockState,
    pub slab: BlockState,
}

impl BlockSlab {
    pub fn new(block: BlockState, slab: BlockState) -> Self {
        Self { block, slab }
    }
}

// #[derive(Clone, Debug)]
// pub struct BlockStairCollection(pub BlockChoice<BlockStair>);

// /// Actually contains a block, slab, and stair
// #[derive(Clone, Debug)]
// pub struct BlockStair {
//     pub block: BlockState,
//     pub slab: BlockState,
//     pub stair: BlockState,
// }

// impl BlockStair {
//     pub fn new(block: BlockState, slab: BlockState, stair: BlockState) -> Self {
//         Self { block, slab, stair }
//     }
// }

/// The `TerrainBlockCollection` struct represents a collection of different types
/// of blocks used in a terrain, such as grass, dirt, stone, and liquid.
/// 
/// Properties:
/// 
/// * `grass`: The `grass` property is of type `BlockCollection`. It represents a
/// collection of blocks that are placed at the top of the terrain.
/// * `dirt`: The `dirt` property is an optional `BlockCollection`. It can either be
/// `Some(BlockCollection)` or `None`. If it is `Some`, then it represents a
/// collection of blocks that are placed 1 and 2 blocks below `grass`. If it is
/// `None`, then the `stone` property is used instead
/// * `stone`: The `stone` property is of type `BlockCollection`. It represents a
/// collection of blocks that take up the majority of the terrain.
/// * `liquid`: The `liquid` property is an optional `BlockCollection` that
/// represents the blocks used for liquid terrain. If the `liquid` property is
/// `Some`, it means that a specific `BlockCollection` is used for liquid terrain.
/// If the `liquid` property is `None`, it means that no liquid terrain is used.
/// Liquid terrain is placed above ground level, and always goes up to the same
/// height. Grass is never placed below liquid terrain.
#[derive(Clone, Debug)]
pub struct TerrainBlockCollection {
    pub grass: BlockCollection,
    /// If None, then stone is used
    pub dirt: Option<BlockCollection>,
    pub stone: BlockCollection,
    pub liquid: Option<BlockCollection>,
}

/// The `BlinkBlockCollection` struct represents a collection of the two types
/// of blocks used for the blink blocks generation.
/// 
/// Properties:
/// 
/// * `on`: The `on` property is of type `BlockCollection`. It represents a
/// collection of blocks that are used when the blink blocks are on.
/// * `off`: The `off` property is of type `BlockCollection`. It represents a
/// collection of blocks that are used when the blink blocks are off.
#[derive(Clone, Debug)]
pub struct BlinkBlockCollection {
    pub on: BlockCollection,
    pub off: BlockCollection,
}

/// The `IndoorBlockCollection` struct represents a collection of different types
/// of blocks used in an indoor area, such as the walls and ceiling of the area,
/// the floor, and the blocks used to create the platforms in the area.
///
/// Properties:
/// 
/// * `walls`: The `walls` property is of type `BlockCollection`. It represents a
/// collection of blocks that are used to create the walls and ceiling of the area.
/// * `floor`: The `floor` property is and optional `BlockCollection`. It can either
/// be `Some(BlockCollection)` or `None`. If it is `Some`, then it represents a
/// collection of blocks that are used to create the floor of the area. If it is
/// `None`, then there is no floor. If the `floor` is of length 1 and the only
/// block in the `floor` is `WATER` or `LAVA`, then `walls` are placed below the
/// `floor` to prevent the player from falling into the liquid and the `platforms`
/// are placed one block lower than normal as the player can't jump out of the
/// liquid.
/// * `platforms`: The `platforms` property is of type `BlockSlabCollection`. It
/// represents a collection of blocks that are used to create the platforms in the
/// area.
#[derive(Clone, Debug)]
pub struct IndoorBlockCollection {
    pub walls: BlockCollection,
    pub floor: Option<BlockCollection>,
    pub platforms: BlockSlabCollection,
}

/// The `CustomBlockCollection` struct represents a pre-defined collection of
/// blocks used in a custom parkour generation as well as a start position and
/// end position.
/// 
/// Properties:
/// 
/// * `blocks`: The `blocks` property is of type `HashMap<BlockPos, BlockState>`.
/// It represents the blocks that are used in the custom parkour generation.
/// * `start_pos`: The `start_pos` property is of type `BlockPos`. It represents
/// the start position of the custom parkour generation.
/// * `end_pos`: The `end_pos` property is of type `BlockPos`. It represents the
/// end position of the custom parkour generation.
#[derive(Clone, Debug)]
pub struct CustomBlockCollection {
    pub blocks: HashMap<BlockPos, BlockState>,
    pub start_pos: BlockPos,
    pub end_pos: BlockPos,
}
