use std::collections::HashMap;

use rand::Rng;
use valence::{math::IVec3, prelude::*};

use crate::{
    generation::{
        block_collection::*,
        generation::ChildGeneration,
        generator::{BlockGenParams, BlockGenerator, GenerateResult},
    },
    line::Line3,
    prediction::prediction_state::PredictionState,
    utils::*,
};

pub struct IndoorGenerator {
    /// Used for walls and ceiling
    pub walls: String,
    /// Used for floor. If `None`, there is no floor. If liquid, then `walls`
    /// is placed below `floor`.
    pub floor: Option<String>,
    /// Used for platforms
    pub platforms: String,
}

impl IndoorGenerator {
    fn get_platform_level(&self) -> i32 {
        match &self.floor {
            Some(_) => 2,
            _ => 0,
        }
    }

    fn generate_walls(
        &self,
        blocks: &mut HashMap<BlockPos, BlockState>,
        size: &IVec3,
        map: &BuiltBlockCollectionMap,
    ) {
        let mut pos = BlockPos::new(0, 0, 0);

        let mut wall_blocks = Vec::new();

        for y in 0..size.y {
            for z in 0..size.z {
                let pos = BlockPos::new(pos.x, pos.y + y, pos.z + z);
                wall_blocks.push((pos, map.get_block(&self.walls)));

                let pos = BlockPos::new(pos.x + size.x - 1, pos.y, pos.z);

                wall_blocks.push((pos, map.get_block(&self.walls)));
            }
        }

        pos.y += size.y;

        for x in 0..size.x {
            for z in 0..size.z {
                let pos = BlockPos::new(pos.x + x, pos.y, pos.z + z);
                wall_blocks.push((pos, map.get_block(&self.walls)));
            }
        }

        blocks.extend(wall_blocks);
    }

    fn generate_floor(
        &self,
        blocks: &mut HashMap<BlockPos, BlockState>,
        size: &IVec3,
        map: &BuiltBlockCollectionMap,
    ) {
        if let Some(floor) = &self.floor {
            let mut pos = BlockPos::new(0, 0, 0);

            let mut floor_blocks = HashMap::new();

            let liquid = map.is_liquid(&floor);

            if liquid {
                for x in 1..size.x - 1 {
                    for z in 0..size.z {
                        let pos = BlockPos::new(pos.x + x, pos.y, pos.z + z);
                        floor_blocks.insert(pos, map.get_block(&self.walls));
                    }
                }

                pos.y += 1;
            }

            for x in 1..size.x - 1 {
                for z in if liquid { 1 } else { 0 }..size.z {
                    let pos = BlockPos::new(pos.x + x, pos.y, pos.z + z);
                    floor_blocks.insert(pos, map.get_block(floor));
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
        map: &BuiltBlockCollectionMap,
    ) -> BlockPos {
        let mut rng = rand::thread_rng();
        // TODO: Improve

        let start = BlockPos::new(rng.gen_range(1..size.x - 1), platform_level, 0);

        if platform_level > 0 {
            for x in 1..size.x - 1 {
                blocks.insert(BlockPos::new(x, 1, 0), map.get_block(&self.walls));
            }
        }

        blocks.insert(start, map.get_block(&self.platforms));

        start
    }

    fn generate_platforms(
        &self,
        size: &IVec3,
        floor_level: i32,
        prev: BlockPos,
        lines: &mut Vec<Line3>,
        children: &mut Vec<ChildGeneration>,
        map: &BuiltBlockCollectionMap,
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

                return self.generate_platforms(size, floor_level, prev, lines, children, map);
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
            HashMap::from([(pos, map.get_block(&self.platforms))]),
            HashMap::new(),
        ));

        lines.append(&mut new_lines);

        self.generate_platforms(size, floor_level, pos, lines, children, map)
    }
}

impl BlockGenerator for IndoorGenerator {
    fn generate(&self, params: &BlockGenParams) -> GenerateResult {
        let map = &params.block_map;
        let mut blocks = HashMap::new();
        let mut rng = rand::thread_rng();

        let mut size: IVec3 = IVec3::new(rng.gen_range(5..=10), 7, rng.gen_range(15..=30));

        let mut lines = Vec::new();

        let mut children = Vec::new();

        let platform_level = self.get_platform_level();
        let start = self.generate_start(&mut blocks, &size, platform_level, &map);
        let end = self.generate_platforms(
            &size,
            platform_level,
            start,
            &mut lines,
            &mut children,
            &map,
        );

        size.z = end.z + 1;

        self.generate_floor(&mut blocks, &size, &map);
        self.generate_walls(&mut blocks, &size, &map);

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
