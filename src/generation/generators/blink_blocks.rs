use std::collections::HashMap;

use rand::Rng;
use valence::{math::*, prelude::*};

use crate::{
    alt_block::*,
    generation::{block_collection::*, generation::ChildGeneration, generator::GenerateResult},
    line::Line3,
    prediction::prediction_state::PredictionState,
    utils::*,
};

pub struct BlinkBlocksGenerator {
    collection: BlinkBlockCollection,
    size: IVec2,
    delay: usize,
    on_index: usize,
    off_index: usize,
}

impl BlinkBlocksGenerator {
    pub fn new(collection: BlinkBlockCollection, size: IVec2) -> Self {
        Self {
            size,
            delay: 25,
            on_index: collection
                .on
                .0
                .blocks
                .get_random_index()
                .expect("No blocks in on collection"),
            off_index: collection
                .off
                .0
                .blocks
                .get_random_index()
                .expect("No blocks in off collection"),
            collection,
        }
    }

    fn get_on_block(&self) -> BlockState {
        if self.collection.on.0.uniform {
            self.collection.on.0.blocks[self.on_index].clone()
        } else {
            self.collection.on.0.blocks.get_random().unwrap().clone()
        }
    }

    fn get_off_block(&self) -> BlockState {
        if self.collection.off.0.uniform {
            self.collection.off.0.blocks[self.off_index].clone()
        } else {
            self.collection.off.0.blocks.get_random().unwrap().clone()
        }
    }

    fn create_on_alt_block(&self) -> AltBlock {
        AltBlock::Tick(
            vec![
                (AltBlockState::Block(self.get_on_block()), self.delay),
                (AltBlockState::SmallBlock(self.get_on_block()), self.delay),
            ],
            0,
        )
    }

    fn create_off_alt_block(&self) -> AltBlock {
        AltBlock::Tick(
            vec![
                (AltBlockState::SmallBlock(self.get_off_block()), self.delay),
                (AltBlockState::Block(self.get_off_block()), self.delay),
            ],
            0,
        )
    }

    fn create_on_child(&self, pos: BlockPos) -> ChildGeneration {
        let mut blocks = HashMap::new();
        let mut alt_blocks = HashMap::new();

        for x in 0..self.size.x {
            for z in 0..self.size.y {
                let block = self.get_on_block();
                let offset = BlockPos::new(x - self.size.x / 2, 0, z);

                blocks.insert(pos + offset, block);
                alt_blocks.insert(pos + offset, self.create_on_alt_block());
            }
        }

        ChildGeneration::new(blocks, alt_blocks)
    }

    fn create_off_child(&self, pos: BlockPos) -> ChildGeneration {
        let mut blocks = HashMap::new();
        let mut alt_blocks = HashMap::new();

        for x in 0..self.size.x {
            for z in 0..self.size.y {
                let block = self.get_off_block();
                let offset = BlockPos::new(x - self.size.x / 2, 0, z);

                blocks.insert(pos + offset, block);
                alt_blocks.insert(pos + offset, self.create_off_alt_block());
            }
        }

        ChildGeneration::new(blocks, alt_blocks)
    }

    fn create_on_off_next_to_each_other_child(&self, pos: BlockPos) -> (ChildGeneration, BlockPos) {
        let off = rand::thread_rng().gen();
        let mut blocks = HashMap::new();
        let mut alt_blocks = HashMap::new();

        for x in 0..self.size.x {
            for z in 0..self.size.y {
                let offset = BlockPos::new(x - self.size.x / 2, 0, z);

                blocks.insert(pos + offset, BlockState::AIR);
                alt_blocks.insert(
                    pos + offset,
                    if off {
                        self.create_off_alt_block()
                    } else {
                        self.create_on_alt_block()
                    },
                );
            }
        }

        let o = (self.size.x + 1) * random_sign();

        for x in 0..self.size.x {
            for z in 0..self.size.y {
                let offset = BlockPos::new(o + x - self.size.x / 2, 0, z);

                blocks.insert(pos + offset, BlockState::AIR);
                alt_blocks.insert(
                    pos + offset,
                    if off {
                        self.create_on_alt_block()
                    } else {
                        self.create_off_alt_block()
                    },
                );
            }
        }

        let pos = pos
            + BlockPos::new(
                if rand::thread_rng().gen() { o } else { 0 },
                0,
                self.size.y - 1,
            );

        (ChildGeneration::new(blocks, alt_blocks), pos)
    }

    pub fn generate(&self, direction: JumpDirection) -> GenerateResult {
        let mut rng = rand::thread_rng();
        let mut children = Vec::new();

        let (mut g, mut pos) = self.create_on_off_next_to_each_other_child(BlockPos::new(0, 0, 0));

        children.push(ChildGeneration {
            reached: true, // First block is always reached
            ..g
        });

        let mut lines = Vec::new();

        for _ in 0..rng.gen_range(1..=5) {
            let mut prediction = PredictionState::running_jump_block(pos, random_yaw());

            let target_y = direction.get_y_offset();

            loop {
                let mut new_prediction = prediction.clone();
                new_prediction.tick();

                lines.push(Line3::new(
                    prediction.pos.as_vec3(),
                    new_prediction.pos.as_vec3(),
                ));

                if new_prediction.vel.y > 0. || new_prediction.pos.y > target_y as f64 {
                    prediction = new_prediction;
                } else {
                    break;
                }
            }

            pos = prediction.get_block_pos();

            (g, pos) = self.create_on_off_next_to_each_other_child(pos);

            children.push(g);
        }

        GenerateResult {
            start: BlockPos::new(0, 0, 0),
            end: pos,
            blocks: HashMap::new(),
            alt_blocks: HashMap::new(),
            lines,
            children,
        }
    }
}
