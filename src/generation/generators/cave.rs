use std::collections::{HashMap, HashSet};

use rand::Rng;
use valence::{math::*, prelude::*};

use crate::{
    generation::{block_collection::*, generation::ChildGeneration, generator::GenerateResult},
    line::Line3,
    prediction::prediction_state::PredictionState,
    utils::*,
};

pub struct CaveGenerator {
    collection: BlockChoice<BlockState>,
    index: usize,
}

impl CaveGenerator {
    pub fn new(collection: BlockChoice<BlockState>) -> Self {
        let i = collection.blocks.get_random_index().unwrap();
        Self {
            collection,
            index: i,
        }
    }

    fn get_block(&self) -> BlockState {
        if self.collection.uniform {
            self.collection.blocks[self.index].clone()
        } else {
            self.collection.blocks.get_random().unwrap().clone()
        }
    }

    pub fn generate(&self) -> GenerateResult {
        let mut rng = rand::thread_rng();

        let mut size: IVec3 = IVec3::new(
            rng.gen_range(10..=20),
            rng.gen_range(12..=18),
            rng.gen_range(15..=60),
        );

        let start = BlockPos::new(size.x / 2, 1, 0);

        let mut lines = Vec::new();

        let mut blocks = HashMap::new();
        let mut children = Vec::new();
        let mut air = HashSet::new();

        let end = self.generate_platforms(
            &mut air,
            &mut children,
            &size,
            start,
            HashSet::from([
                IVec2::new(start.x - 1, start.z - 1),
                IVec2::new(start.x, start.z - 1),
                IVec2::new(start.x + 1, start.z - 1),
                IVec2::new(start.x - 1, start.z),
                IVec2::new(start.x, start.z),
                IVec2::new(start.x + 1, start.z),
                IVec2::new(start.x - 1, start.z + 1),
                IVec2::new(start.x, start.z + 1),
                IVec2::new(start.x + 1, start.z + 1),
            ]),
            1,
            &mut lines,
        );

        size.z = end.z + 1;

        self.fill(&mut blocks, &size);

        for air in air {
            blocks.insert(air, BlockState::AIR);
        }

        blocks.insert(start, self.get_block());

        (start, blocks, end, lines, children)
    }

    fn fill(&self, blocks: &mut HashMap<BlockPos, BlockState>, size: &IVec3) {
        let pos = BlockPos::new(0, 0, 0);

        for x in -1..size.x + 1 {
            for y in -1..size.y + 1 {
                for z in 0..size.z {
                    let pos = BlockPos::new(pos.x + x, pos.y + y, pos.z + z);
                    blocks.insert(pos, self.get_block());
                }
            }
        }
    }

    fn generate_platforms(
        &self,
        air: &mut HashSet<BlockPos>,
        children: &mut Vec<ChildGeneration>,
        size: &IVec3,
        prev: BlockPos,
        prev_xz_air: HashSet<IVec2>,
        floor_level: i32,
        lines: &mut Vec<Line3>,
    ) -> BlockPos {
        // FIXME: Sometimes, next to the platform, there is an unescapable hole.
        if prev.z >= size.z - 1 {
            return prev;
        }

        let mut rng = rand::thread_rng();

        let mut blocks = HashMap::new();

        let (min_yaw, max_yaw) = get_min_max_yaw(prev, size);

        let yaw = -rng.gen_range(min_yaw..=max_yaw);

        let mut prediction = PredictionState::running_jump_block(prev, yaw);

        let target_y = if prev.y <= 3 {
            prev.y + 2
        } else if prev.y >= size.y - 5 {
            prev.y
        } else {
            prev.y + rng.gen_range(0..=2)
        };

        let mut intersected_blocks = HashSet::new();

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

                return self.generate_platforms(
                    air,
                    children,
                    size,
                    prev,
                    prev_xz_air,
                    floor_level,
                    lines,
                );
            }

            if new_prediction.vel.y > 0. || new_prediction.pos.y > target_y as f64 {
                new_lines.push(Line3::new(
                    prediction.pos.as_vec3(),
                    new_prediction.pos.as_vec3(),
                ));

                prediction = new_prediction;
                let blocks = prediction.get_intersected_blocks();
                intersected_blocks.extend(blocks);
            } else {
                break;
            }
        }
        let pos = prediction.get_block_pos();

        intersected_blocks.retain(|b| {
            prediction.yaw >= 0. && b.x >= pos.x || prediction.yaw <= 0. && b.x <= pos.x
        });

        let mut floor_level = floor_level;

        let mut all_xz = prev_xz_air.clone();
        let xz_air: HashSet<_> = intersected_blocks
            .iter()
            .map(|b| IVec2::new(b.x, b.z))
            .collect();
        all_xz.extend(xz_air.clone());

        let mut no_air = HashSet::new();

        if !(all_xz.contains(&IVec2::new(prev.x - 1, prev.z - 1))
            && all_xz.contains(&IVec2::new(prev.x - 1, prev.z))
            && all_xz.contains(&IVec2::new(prev.x - 1, prev.z + 1))
            || all_xz.contains(&IVec2::new(prev.x + 1, prev.z - 1))
                && all_xz.contains(&IVec2::new(prev.x + 1, prev.z))
                && all_xz.contains(&IVec2::new(prev.x + 1, prev.z + 1)))
        {
            floor_level = floor_level.max(prev.y - 1);
            floor_level = floor_level.min(target_y - 2);

            let mut prev_child = children.pop().expect("No children");

            let mut blocks = prev_child.blocks;

            for z in 1..=prev.y - floor_level {
                for y in floor_level..=prev.y - z {
                    blocks.insert(BlockPos::new(prev.x, y, prev.z + z), self.get_block());
                    blocks.insert(
                        BlockPos::new(prev.x + z - 1, y, prev.z + 1),
                        self.get_block(),
                    );
                    blocks.insert(
                        BlockPos::new(prev.x - z + 1, y, prev.z + 1),
                        self.get_block(),
                    );
                }
            }

            // FIXME: Very much not ideal
            for y in 1..floor_level {
                no_air.insert(BlockPos::new(prev.x - 1, y, prev.z));
                no_air.insert(BlockPos::new(prev.x + 1, y, prev.z));
                no_air.insert(BlockPos::new(prev.x - 1, y, prev.z + 1));
                no_air.insert(BlockPos::new(prev.x, y, prev.z + 1));
                no_air.insert(BlockPos::new(prev.x + 1, y, prev.z + 1));
            }

            prev_child.blocks = blocks;

            children.push(prev_child);
        } else {
            floor_level = floor_level.min(target_y - 2);
            floor_level = floor_level.max(target_y - 3);
        }

        for b in intersected_blocks {
            if no_air.contains(&b) {
                continue;
            }
            for y in floor_level..=b.y {
                air.insert(BlockPos::new(b.x, y, b.z));
            }
        }

        blocks.insert(pos, self.get_block());

        for y in 1..pos.y {
            blocks.insert(BlockPos::new(pos.x, y, pos.z), self.get_block());
        }

        lines.extend(new_lines);

        children.push(ChildGeneration::new(blocks, HashMap::new()));

        self.generate_platforms(air, children, size, pos, xz_air, floor_level, lines)
    }
}
