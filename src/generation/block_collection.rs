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
    pub uniform: bool, // TODO: I don't like this. I sometimes even ignore it. There has to be a better way.
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

/// The `BlockCollectionMap` struct represents a collection of an arbitrary number
/// of `BlockCollection`s with a name associated with each one. This is used to
/// store the different types of blocks used in a generation.
///
/// If you require different shapes of the same type of block (e.g. full blocks,
/// slabs, and stairs), then the keys should be of the form
/// `"<name>_<shape>"`, where `<name>` is the name of the block and `<shape>` is
/// the shape of the block. For example, if you have a block called `stone` and
/// you want to use full blocks, slabs, and stairs, then you should use the keys
/// `"stone_full"`, `"stone_slab"`, and `"stone_stair"`.
///
/// If only one shape of a block is required, then the key should just be the name
/// of the block. For example, if you have a block called `grass`, then you should
/// use the key `"grass"`.
///
/// Properties:
///
/// * `collections`: The `collections` property is a `HashMap<String, BlockCollection>`.
/// It maps a name to a `BlockCollection`.
#[derive(Clone, Debug)]
pub struct BlockCollectionMap {
    pub collections: HashMap<String, BlockCollection>,
}

impl BlockCollectionMap {
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, collection: BlockCollection) {
        self.collections.insert(name, collection);
    }

    pub fn build(self) -> BuiltBlockCollectionMap {
        let mut collections = HashMap::new();
        for (name, collection) in self.collections {
            let index = if collection.0.uniform {
                collection
                    .0
                    .blocks
                    .get_random_index()
                    .expect("No blocks in collection")
            } else {
                0
            };
            collections.insert(name, (collection, index));
        }
        BuiltBlockCollectionMap { collections }
    }
}

impl From<Vec<(String, BlockCollection)>> for BlockCollectionMap {
    fn from(collections: Vec<(String, BlockCollection)>) -> Self {
        let mut map = Self::new();
        for (name, collection) in collections {
            map.add(name, collection);
        }
        map
    }
}

#[derive(Clone, Debug)]
pub struct BuiltBlockCollectionMap {
    pub collections: HashMap<String, (BlockCollection, usize)>,
}

impl BuiltBlockCollectionMap {
    pub fn get_block(&self, name: &str) -> Option<BlockState> {
        let (collection, index) = self.collections.get(name)?;
        if collection.0.uniform {
            Some(collection.0.blocks[*index].clone())
        } else {
            Some(collection.0.blocks.get_random().unwrap().clone())
        }
    }
}

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
