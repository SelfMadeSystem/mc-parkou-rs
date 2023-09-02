use valence::prelude::*;

/// A bunch of blocks that are spawned at once.
pub struct BunchOfBlocks {
    /// The blocks that are spawned.
    blocks: Vec<(BlockPos, BlockState)>,
    /// The end position to expect the player to be at.
    pub end_pos: BlockPos,
}

impl BunchOfBlocks {
    pub fn single(pos: BlockPos, block: BlockState) -> Self {
        Self {
            blocks: vec![(pos, block)],
            end_pos: pos,
        }
    }

    pub fn island(pos: BlockPos, size: i32) -> Self {
        let mut blocks = vec![];

        for z in 0..=size * 2 {
            // the min to max range of x values so that the island is a circle
            // Since z starts at 0 and goes to size * 2, we need to subtract size
            // to make it go from -size to size.
            let s = {
                let size = size as f32;
                let mut z = z as f32;
                if z == 0. {
                    z = 0.25;
                } else if z == size * 2. {
                    z = size * 2. - 0.25;
                }
                ((size * size - (z - size) * (z - size)) as f32).sqrt().round() as i32
            };
            for x in -s..=s {
                let pos = BlockPos {
                    x: pos.x + x,
                    y: pos.y,
                    z: pos.z + z,
                };
                blocks.push((
                    pos,
                    if x == 0 && z == 0 {
                        BlockState::STONE
                    } else {
                        BlockState::GRASS_BLOCK
                    },
                ));
            }
        }

        Self {
            blocks,
            end_pos: BlockPos {
                x: pos.x,
                y: pos.y,
                z: pos.z + size * 2,
            },
        }
    }

    pub fn place(&self, world: &mut ChunkLayer) {
        for (pos, block) in &self.blocks {
            world.set_block(*pos, *block);
        }
    }

    pub fn remove(&self, world: &mut ChunkLayer) {
        for (pos, _) in &self.blocks {
            world.set_block(*pos, BlockState::AIR);
        }
    }

    /// Returns true if the player has reached any of the blocks.
    pub fn has_reached(&self, pos: Position) -> bool {
        let pos = BlockPos::new(
            (pos.0.x - 0.5).round() as i32,
            pos.0.y as i32 - 1,
            (pos.0.z - 0.5).round() as i32,
        );

        self.blocks.iter().any(|(block_pos, _)| *block_pos == pos)
    }
}
