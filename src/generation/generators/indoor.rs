use std::collections::HashMap;

use rand::Rng;
use valence::{math::IVec3, prelude::*};

use crate::{
    generation::{
        block_collection::*,
        generation::ChildGeneration,
        generator::{BlockGenerator, GenerateResult},
    },
    line::Line3,
    prediction::prediction_state::PredictionState,
    utils::*,
};

pub struct IndoorGenerator {
    collection: IndoorBlockCollection,
    wall_index: usize,
    floor_index: usize,
    platform_index: usize,
}

impl IndoorGenerator {
    pub fn new(collection: IndoorBlockCollection) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            wall_index: rng.gen_range(0..collection.walls.0.blocks.len()),
            floor_index: match &collection.floor {
                Some(floor) => rng.gen_range(0..floor.0.blocks.len()),
                None => 0,
            },
            platform_index: rng.gen_range(0..collection.platforms.0.blocks.len()),
            collection,
        }
    }

    fn get_wall(&self) -> BlockState {
        if self.collection.walls.0.uniform {
            self.collection.walls.0.blocks[self.wall_index].clone()
        } else {
            self.collection.walls.0.blocks.get_random().unwrap().clone()
        }
    }

    fn get_floor(&self) -> BlockState {
        match &self.collection.floor {
            Some(floor) => {
                if floor.0.uniform {
                    floor.0.blocks[self.floor_index].clone()
                } else {
                    floor.0.blocks.get_random().unwrap().clone()
                }
            }
            None => BlockState::AIR,
        }
    }

    fn get_platform_block_slab(&self) -> BlockSlab {
        let platform = if self.collection.platforms.0.uniform {
            self.collection.platforms.0.blocks[self.platform_index].clone()
        } else {
            self.collection
                .platforms
                .0
                .blocks
                .get_random()
                .unwrap()
                .clone()
        };
        platform
    }

    fn get_platform(&self) -> (BlockState, BlockState) {
        let platform = self.get_platform_block_slab();
        (platform.block, platform.slab)
    }

    fn get_platform_level(&self) -> i32 {
        match &self.collection.floor {
            Some(_) => 2,
            _ => 0,
        }
    }

    fn generate_walls(&self, blocks: &mut HashMap<BlockPos, BlockState>, size: &IVec3) {
        let mut pos = BlockPos::new(0, 0, 0);

        let mut wall_blocks = Vec::new();

        for y in 0..size.y {
            for z in 0..size.z {
                let pos = BlockPos::new(pos.x, pos.y + y, pos.z + z);
                wall_blocks.push((pos, self.get_wall()));

                let pos = BlockPos::new(pos.x + size.x - 1, pos.y, pos.z);

                wall_blocks.push((pos, self.get_wall()));
            }
        }

        pos.y += size.y;

        for x in 0..size.x {
            for z in 0..size.z {
                let pos = BlockPos::new(pos.x + x, pos.y, pos.z + z);
                wall_blocks.push((pos, self.get_wall()));
            }
        }

        blocks.extend(wall_blocks);
    }

    fn generate_floor(&self, blocks: &mut HashMap<BlockPos, BlockState>, size: &IVec3) {
        if let Some(floor) = &self.collection.floor {
            let mut pos = BlockPos::new(0, 0, 0);

            let mut floor_blocks = HashMap::new();

            let liquid = floor.0.blocks.len() == 1 && floor.0.blocks[0].is_liquid();

            if liquid {
                for x in 1..size.x - 1 {
                    for z in 0..size.z {
                        let pos = BlockPos::new(pos.x + x, pos.y, pos.z + z);
                        floor_blocks.insert(pos, self.get_wall());
                    }
                }

                pos.y += 1;
            }

            for x in 1..size.x - 1 {
                for z in if liquid { 1 } else { 0 }..size.z {
                    let pos = BlockPos::new(pos.x + x, pos.y, pos.z + z);
                    floor_blocks.insert(pos, self.get_floor());
                }
            }

            blocks.extend(floor_blocks);
        }
    }

    fn generate_start(
        &self,
        blocks: &mut HashMap<BlockPos, BlockState>,
        size: &IVec3,
        platform_level: i32,
    ) -> BlockPos {
        let mut rng = rand::thread_rng();
        // TODO: Improve

        let start = BlockPos::new(rng.gen_range(1..size.x - 1), platform_level, 0);

        if platform_level > 0 {
            for x in 1..size.x - 1 {
                blocks.insert(BlockPos::new(x, 1, 0), self.get_wall());
            }
        }

        blocks.insert(start, self.get_platform().0);

        start
    }

    fn generate_platforms(
        &self,
        size: &IVec3,
        floor_level: i32,
        prev: BlockPos,
        lines: &mut Vec<Line3>,
        children: &mut Vec<ChildGeneration>,
    ) -> BlockPos {
        if prev.z >= size.z - 1 {
            return prev;
        }

        let mut rng = rand::thread_rng();

        let (min_yaw, max_yaw) = get_min_max_yaw(prev, size);

        let yaw = -rng.gen_range(min_yaw..=max_yaw);
        let mut prediction = PredictionState::running_jump_block(prev, yaw);

        let mut new_lines = Vec::new();

        loop {
            let mut new_prediction = prediction.clone();
            new_prediction.tick();

            // uncertain if this will cause an infinite loop
            // hopefully not
            if new_prediction.pos.x < 1. || new_prediction.pos.x >= size.x as f64 - 1. {
                eprintln!("Platform out of bounds. yaw: {:.2} min_yaw: {:.2}, max_yaw: {:.2}, prev: {:?}, size: {:?}", yaw.to_degrees(), min_yaw.to_degrees(), max_yaw.to_degrees(), prev, size);
                eprintln!("{}", new_prediction.pos.x);
                eprintln!(
                    "{}",
                    new_prediction
                        .pos
                        .with_y(0.)
                        .distance(prev.to_vec3().as_dvec3().with_y(0.))
                );
                eprintln!("{}", new_prediction.vel.x);
                // try again. TODO: Improve

                return self.generate_platforms(size, floor_level, prev, lines, children);
            }

            if new_prediction.vel.y > 0. || new_prediction.pos.y > floor_level as f64 + 1. {
                new_lines.push(Line3::new(
                    prediction.pos.as_vec3(),
                    new_prediction.pos.as_vec3(),
                ));

                prediction = new_prediction;
            } else {
                break;
            }
        }

        let pos = prediction.get_block_pos();

        children.push(ChildGeneration::new(
            HashMap::from([(pos, self.get_platform().0)]),
            HashMap::new(),
        ));

        lines.append(&mut new_lines);

        self.generate_platforms(size, floor_level, pos, lines, children)
    }
}

impl BlockGenerator for IndoorGenerator {
    fn generate(&self) -> GenerateResult {
        let mut blocks = HashMap::new();
        let mut rng = rand::thread_rng();

        let mut size: IVec3 = IVec3::new(rng.gen_range(5..=10), 7, rng.gen_range(15..=30));

        let mut lines = Vec::new();

        let mut children = Vec::new();

        let platform_level = self.get_platform_level();
        let start = self.generate_start(&mut blocks, &size, platform_level);
        let end = self.generate_platforms(&size, platform_level, start, &mut lines, &mut children);

        size.z = end.z + 1;

        self.generate_floor(&mut blocks, &size);
        self.generate_walls(&mut blocks, &size);

        GenerateResult {
            start,
            end,
            blocks,
            alt_blocks: HashMap::new(),
            lines,
            children,
        }
    }
}
