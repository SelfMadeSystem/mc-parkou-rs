use lazy_static::lazy_static;
use valence::BlockState;

lazy_static! {
    pub static ref SLAB_TYPES: Vec<(BlockState, BlockState)> = vec![
        (BlockState::STONE, BlockState::STONE_SLAB),
        (BlockState::COBBLESTONE, BlockState::COBBLESTONE_SLAB),
        (
            BlockState::MOSSY_COBBLESTONE,
            BlockState::MOSSY_COBBLESTONE_SLAB,
        ),
        (BlockState::STONE_BRICKS, BlockState::STONE_BRICK_SLAB),
        (BlockState::OAK_PLANKS, BlockState::OAK_SLAB),
        (BlockState::SPRUCE_PLANKS, BlockState::SPRUCE_SLAB),
    ];

    pub static ref GENERIC_BLOCK_TYPES: Vec<BlockState> = vec![
        BlockState::GRASS_BLOCK,
        BlockState::OAK_LOG,
        BlockState::BIRCH_LOG,
        BlockState::OAK_LEAVES,
        BlockState::BIRCH_LEAVES,
        BlockState::DIRT,
        BlockState::MOSS_BLOCK,
    ];

    pub static ref UNDERGROUND_BLOCK_TYPES: Vec<BlockState> = vec![
        BlockState::STONE,
        BlockState::COBBLESTONE,
        BlockState::MOSSY_COBBLESTONE,
        BlockState::ANDESITE,
        BlockState::DIORITE,
        BlockState::GRANITE,
        BlockState::GRAVEL,
    ];
}
