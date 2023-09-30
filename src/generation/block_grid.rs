use std::collections::HashMap;

use valence::prelude::*;

use crate::utils::*;

use super::block_collection::BuiltBlockCollectionMap;

#[derive(Clone, Debug)]
pub struct BlockProperties {
    pub name: String,
    pub properties: Vec<PropNameValue>,
}

impl BlockProperties {
    pub fn new(name: String, properties: Vec<PropNameValue>) -> Self {
        Self { name, properties }
    }

    pub fn get_block(&self, block_map: &BuiltBlockCollectionMap) -> BlockState {
        let mut block = block_map.get_block(&self.name);
        for (name, value) in &self.properties {
            block = block.set(*name, *value);
        }
        block
    }

    pub fn rotate_cw(&self) -> Self {
        let mut properties = self.properties.clone();

        for prop in &mut properties {
            *prop = prop_nv_rotate_cw(prop);
        }

        Self {
            name: self.name.clone(),
            properties,
        }
    }

    pub fn flip_x(&self) -> Self {
        let mut properties = self.properties.clone();

        for prop in &mut properties {
            *prop = prop_nv_flip_x(prop);
        }

        Self {
            name: self.name.clone(),
            properties,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BlockGrid {
    pub blocks: HashMap<BlockPos, BlockProperties>,
}

impl BlockGrid {
    pub fn new(blocks: HashMap<BlockPos, BlockProperties>) -> Self {
        Self { blocks }
    }
}

impl From<HashMap<BlockPos, BlockProperties>> for BlockGrid {
    fn from(blocks: HashMap<BlockPos, BlockProperties>) -> Self {
        Self::new(blocks)
    }
}

impl From<Vec<(BlockPos, BlockProperties)>> for BlockGrid {
    fn from(blocks: Vec<(BlockPos, BlockProperties)>) -> Self {
        Self::new(blocks.into_iter().collect())
    }
}

impl<const N: usize> From<[(BlockPos, BlockProperties); N]> for BlockGrid {
    fn from(blocks: [(BlockPos, BlockProperties); N]) -> Self {
        Self::new(blocks.iter().cloned().collect())
    }
}
