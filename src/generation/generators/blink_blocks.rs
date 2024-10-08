use std::collections::HashMap;

use rand::Rng;
use valence::{math::*, prelude::*};

use crate::{
    alt_block::*,
    generation::{
        block_collection::*,
        generation::ChildGeneration,
        generator::{BlockGenParams, BlockGenerator, GenerateResult},
    },
    line::Line3,
    prediction::prediction_state::PredictionState,
    utils::*,
};

pub struct BlinkBlocksGenerator {
    pub on: String,
    pub off: String,
    pub size: IVec2,
    pub delay: usize,
    pub overlap: usize,
}

impl BlinkBlocksGenerator {
    fn create_on_alt_block(&self, map: &BuiltBlockCollectionMap) -> AltBlock {
        AltBlock::Tick(
            vec![
                (
                    AltBlockState::Block(map.get_block(&self.on)),
                    self.delay + self.overlap * 2,
                ),
                (
                    AltBlockState::SmallBlock(map.get_block(&self.on)),
                    self.delay,
                ),
            ],
            self.overlap,
        )
    }

    fn create_off_alt_block(&self, map: &BuiltBlockCollectionMap) -> AltBlock {
        AltBlock::Tick(
            vec![
                (
                    AltBlockState::SmallBlock(map.get_block(&self.off)),
                    self.delay,
                ),
                (
                    AltBlockState::Block(map.get_block(&self.off)),
                    self.delay + self.overlap * 2,
                ),
            ],
            0,
        )
    }

    fn create_on_off_next_to_each_other_child(
        &self,
        pos: BlockPos,
        map: &BuiltBlockCollectionMap,
    ) -> (ChildGeneration, BlockPos) {
        let off = rand::thread_rng().gen();
        let mut blocks = HashMap::new();
        let mut alt_blocks = HashMap::new();

        for x in 0..self.size.x {
            for z in 0..self.size.y {
                let offset = BlockPos::new(x - self.size.x / 2, 0, z);

                blocks.insert(pos + offset.as_ivec3(), BlockState::AIR);
                alt_blocks.insert(
                    pos + offset.as_ivec3(),
                    if off {
                        self.create_off_alt_block(map)
                    } else {
                        self.create_on_alt_block(map)
                    },
                );
            }
        }

        let o = (self.size.x + 1) * random_sign();

        for x in 0..self.size.x {
            for z in 0..self.size.y {
                let offset = BlockPos::new(o + x - self.size.x / 2, 0, z);

                blocks.insert(pos + offset.as_ivec3(), BlockState::AIR);
                alt_blocks.insert(
                    pos + offset.as_ivec3(),
                    if off {
                        self.create_on_alt_block(map)
                    } else {
                        self.create_off_alt_block(map)
                    },
                );
            }
        }

        let pos = pos
            + IVec3::new(
                if rand::thread_rng().gen() { o } else { 0 },
                0,
                self.size.y - 1,
            );

        (ChildGeneration::blocks_alt_blocks(blocks, alt_blocks), pos)
    }
}

impl BlockGenerator for BlinkBlocksGenerator {
    fn generate(&self, params: &BlockGenParams) -> GenerateResult {
        let direction = params.direction;
        let map = &params.block_map;
        let mut rng = rand::thread_rng();
        let mut children = Vec::new();

        let (mut g, mut pos) =
            self.create_on_off_next_to_each_other_child(BlockPos::new(0, 0, 0), &map);

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

            (g, pos) = self.create_on_off_next_to_each_other_child(pos, &map);

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
