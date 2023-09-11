use std::collections::{HashMap, HashSet};

use crate::{line::Line3, prediction::prediction_state::PredictionState, utils::*};

use super::{block_collection::*, generation::Generation, theme::GenerationTheme};
use rand::Rng;
use valence::{layer::chunk::IntoBlock, math::IVec3, prelude::*};

/// The `GenerationType` enum represents the different types of parkour generations
/// that can be used.
///
/// Variants:
/// * `Single`: The `Single` variant represents a single block.
/// * `Slime`: The `Slime` variant represents a slime block.
/// * `Ramp`: The `Ramp` variant represents blocks and slabs that are used to create
/// a ramp.
/// * `Island`: The `Island` variant represents blocks that are used to create an
/// island.
/// * `Bridge`: The `Bridge` variant represents blocks and slabs that are used to
/// create a bridge as well as wall blocks.
/// * `Indoor`: The `Indoor` variant represents blocks that are used to create an
/// indoor area.
/// * `Cave`: The `Cave` variant represents blocks that are used to create a cave.
/// * `Custom`: The `Custom` variant represents a custom parkour generation. It has
/// preset blocks, a start position, and an end position.
#[derive(Clone, Debug)]
pub enum GenerationType {
    Single(BlockCollection),
    // Slime,
    Ramp(BlockSlabCollection),
    // Island(TerrainBlockCollection),
    // Bridge(BridgeBlockCollection),
    Indoor(IndoorBlockCollection),
    Cave(BlockCollection),
    // Custom(CustomGeneration),
}

/// The `Generator` struct represents a parkour generator.
///
/// Properties:
///
/// * `theme`: The `theme` property is of type `GenerationTheme`. It represents the
/// theme of the parkour generator.
/// * `type`: The `type` property is of type `GenerationType`. It represents the
/// type of parkour generation that is used.
/// * `start`: The `start` property is of type `BlockPos`. It represents the start
/// position of the parkour generation.
#[derive(Clone, Debug)]
pub struct Generator {
    pub theme: GenerationTheme,
    pub generation_type: GenerationType,
    pub start: BlockPos,
}

impl Generator {
    pub fn first_in_generation(start: BlockPos, theme: &GenerationTheme) -> Generation {
        let theme = theme.clone();
        let s = Self {
            generation_type: theme.generation_types[0].clone(),
            theme,
            start: BlockPos::new(0, 0, 0),
        };

        let yaw = random_yaw();

        let mut g = s.generate(JumpDirection::DoesntMatter, yaw, Vec::new()); // no lines for first generation

        g.offset = start;
        g.end_state = PredictionState::running_jump_block(start, yaw);

        g
    }

    pub fn next_in_generation(
        direction: JumpDirection,
        theme: &GenerationTheme,
        generation: &Generation,
    ) -> Generation {
        let theme = theme.clone();
        let mut state = generation.end_state.clone();
        let mut lines = Vec::new();
        let mut rng = rand::thread_rng();

        let target_y = match direction {
            JumpDirection::Up => state.pos.y as i32 + 1,
            JumpDirection::Down => state.pos.y as i32 - rng.gen_range(1..=2),
            JumpDirection::DoesntMatter => state.pos.y as i32 + rng.gen_range(-1..=1),
        } as f64;

        let g = loop {
            let mut new_state = state.clone();
            new_state.tick();

            if new_state.vel.y > 0. || new_state.pos.y > target_y {
                lines.push(Line3::new(state.pos.as_vec3(), new_state.pos.as_vec3()));
                state = new_state;
            } else {
                break Self {
                    generation_type: theme.get_random_generation_type(),
                    theme,
                    start: state.get_block_pos(),
                };
            }
        };

        g.generate(direction, generation.end_state.yaw, lines)
    }

    pub fn generate(&self, direction: JumpDirection, yaw: f32, lines: Vec<Line3>) -> Generation {
        let mut blocks = Vec::new();
        let mut offset: BlockPos = self.start;
        let end_state: PredictionState;

        match &self.generation_type {
            GenerationType::Single(BlockCollection(collection)) => {
                blocks.push((
                    BlockPos::new(0, 0, 0),
                    collection
                        .blocks
                        .get_random()
                        .expect("No blocks in block collection")
                        .into_block(),
                ));
                end_state = PredictionState::running_jump_block(self.start, random_yaw());
            }
            GenerationType::Ramp(BlockSlabCollection(collection)) => {
                // TODO: Not great. Should be a better way to do this.
                let index = collection.blocks.get_random_index().unwrap();
                let uniform = collection.uniform;
                let mut rng = rand::thread_rng();
                let new_yaw = random_yaw();

                let height = ((yaw - new_yaw).abs()).round() as i32 + 1;
                let down = match direction {
                    JumpDirection::Up => true,
                    JumpDirection::Down => false,
                    JumpDirection::DoesntMatter => rng.gen(),
                };

                let yaw_change = (new_yaw - yaw) / height as f32;

                let get_block_slab = || {
                    if uniform {
                        collection.blocks[index].clone()
                    } else {
                        collection.blocks.get_random().unwrap().clone()
                    }
                };

                let get_block = || get_block_slab().block.into_block();

                let get_slab = || {
                    let slab = get_block_slab().slab;
                    (
                        slab.into_block(),
                        slab.set(PropName::Type, PropValue::Top).into_block(),
                    )
                };

                let mut pos = Vec3::new(0., 0., 0.);

                let mut curr_yaw = yaw;

                let get_pos_left =
                    |pos: Vec3, yaw: f32| pos + (Vec3::new(yaw.cos(), 0., yaw.sin()) * 2f32.sqrt());

                let get_pos_right = |pos: Vec3, yaw: f32| {
                    pos + (Vec3::new(-yaw.cos(), 0., -yaw.sin()) * 2f32.sqrt())
                };

                let mut block_map = HashMap::new();

                for _ in 0..height {
                    let left = get_pos_left(pos, curr_yaw);
                    let right = get_pos_right(pos, curr_yaw);

                    for b in get_blocks_between(left, right) {
                        block_map.entry(b).or_insert(get_block());
                    }

                    pos.x -= (curr_yaw.sin() * 2f32.sqrt()).clamp(-1., 1.);
                    pos.z += (curr_yaw.cos() * 2f32.sqrt()).clamp(-1., 1.);
                    pos = pos.round();

                    if !down {
                        pos.y += 1.;
                    }

                    let left = get_pos_left(pos, curr_yaw);
                    let right = get_pos_right(pos, curr_yaw);

                    for b in get_blocks_between(left, right) {
                        let c = BlockPos::new(b.x, b.y - 1, b.z);
                        if !block_map.contains_key(&c) {
                            let (slab, top) = get_slab();
                            block_map.entry(c).or_insert(top);
                            block_map.entry(b).or_insert(slab);
                        }
                    }

                    if down {
                        pos.y -= 1.;
                    }

                    curr_yaw += yaw_change;

                    pos.x -= (curr_yaw.sin() * 2f32.sqrt()).clamp(-1., 1.);
                    pos.z += (curr_yaw.cos() * 2f32.sqrt()).clamp(-1., 1.);
                    pos = pos.round();
                }

                let left = get_pos_left(pos, curr_yaw);
                let right = get_pos_right(pos, curr_yaw);

                for b in get_blocks_between(left, right) {
                    block_map.entry(b).or_insert(get_block());
                }
                block_map.entry(pos.as_block_pos()).or_insert(get_block());

                for (pos, block) in block_map {
                    blocks.push((pos, block));
                }

                end_state = PredictionState::running_jump_block(
                    self.start + pos.round().as_block_pos(),
                    new_yaw,
                );
            }
            GenerationType::Indoor(collection) => {
                let indoor = IndoorGenerator::new(collection.clone());

                let (start, bloccs, end) = indoor.generate();

                offset = offset - start;
                blocks = bloccs;
                end_state = PredictionState::running_jump_block(offset + end, random_yaw_dist(30.));
                // walls can be in the way
            }
            GenerationType::Cave(BlockCollection(collection)) => {
                let cave = CaveGenerator::new(collection.clone());

                let (start, bloccs, end) = cave.generate();

                offset = offset - start;
                blocks = bloccs;
                end_state = PredictionState::running_jump_block(offset + end, random_yaw_dist(30.));
            }
        }

        Generation::new(blocks, offset, end_state, lines)
    }
}

struct IndoorGenerator {
    // TODO: NestedGenerator or something idk
    // TODO: Integrate with the combo system
    collection: IndoorBlockCollection,
    wall_index: usize,
    floor_index: usize,
    platform_index: usize,
}

impl IndoorGenerator {
    fn new(collection: IndoorBlockCollection) -> Self {
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

    fn get_wall(&self) -> Block {
        let wall = if self.collection.walls.0.uniform {
            self.collection.walls.0.blocks[self.wall_index].clone()
        } else {
            self.collection.walls.0.blocks.get_random().unwrap().clone()
        };
        wall.into_block()
    }

    fn get_floor(&self) -> Block {
        match &self.collection.floor {
            Some(floor) => {
                let floor = if floor.0.uniform {
                    floor.0.blocks[self.floor_index].clone()
                } else {
                    floor.0.blocks.get_random().unwrap().clone()
                };
                floor.into_block()
            }
            None => BlockState::AIR.into_block(),
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

    fn get_platform(&self) -> (Block, Block) {
        let platform = self.get_platform_block_slab();
        (platform.block.into_block(), platform.slab.into_block())
    }

    fn generate(&self) -> (BlockPos, Vec<(BlockPos, Block)>, BlockPos) {
        let mut blocks = Vec::new();
        let mut rng = rand::thread_rng();

        let mut size: IVec3 = IVec3::new(rng.gen_range(5..=10), 7, rng.gen_range(15..=30));

        let platform_level = self.get_platform_level();
        let start = self.generate_start(&mut blocks, &size, platform_level);
        let end = self.generate_platforms(&mut blocks, &size, platform_level, start);

        size.z = end.z + 1;

        self.generate_floor(&mut blocks, &size);
        self.generate_walls(&mut blocks, &size);

        (start, blocks, end)
    }

    fn get_platform_level(&self) -> i32 {
        match &self.collection.floor {
            Some(_) => 2,
            _ => 0,
        }
    }

    fn generate_walls(&self, blocks: &mut Vec<(BlockPos, Block)>, size: &IVec3) {
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

        blocks.append(&mut wall_blocks);
    }

    fn generate_floor(&self, blocks: &mut Vec<(BlockPos, Block)>, size: &IVec3) {
        if let Some(floor) = &self.collection.floor {
            let mut pos = BlockPos::new(0, 0, 0);

            let mut floor_blocks = Vec::new();

            let liquid = floor.0.blocks.len() == 1 && floor.0.blocks[0].is_liquid();

            if liquid {
                for x in 1..size.x - 1 {
                    for z in 0..size.z {
                        let pos = BlockPos::new(pos.x + x, pos.y, pos.z + z);
                        floor_blocks.push((pos, self.get_wall()));
                    }
                }

                pos.y += 1;
            }

            for x in 1..size.x - 1 {
                for z in if liquid { 1 } else { 0 }..size.z {
                    let pos = BlockPos::new(pos.x + x, pos.y, pos.z + z);
                    floor_blocks.push((pos, self.get_floor()));
                }
            }

            blocks.append(&mut floor_blocks);
        }
    }

    fn generate_start(
        &self,
        blocks: &mut Vec<(BlockPos, Block)>,
        size: &IVec3,
        platform_level: i32,
    ) -> BlockPos {
        let mut rng = rand::thread_rng();
        // TODO: Improve

        let start = BlockPos::new(rng.gen_range(1..size.x - 1), platform_level, 0);

        if platform_level > 0 {
            for x in 1..size.x - 1 {
                blocks.push((BlockPos::new(x, 1, 0), self.get_wall()));
            }
        }

        blocks.push((start, self.get_platform().0));

        start
    }

    fn generate_platforms(
        &self,
        blocks: &mut Vec<(BlockPos, Block)>,
        size: &IVec3,
        floor_level: i32,
        prev: BlockPos,
    ) -> BlockPos {
        if prev.z >= size.z - 1 {
            return prev;
        }

        let mut rng = rand::thread_rng();

        let (min_yaw, max_yaw) = get_min_max_yaw(prev, size);

        let yaw = -rng.gen_range(min_yaw..=max_yaw);
        let mut prediction = PredictionState::running_jump_block(prev, yaw);

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

                return self.generate_platforms(blocks, size, floor_level, prev);
            }

            if new_prediction.vel.y > 0. || new_prediction.pos.y > floor_level as f64 + 1. {
                prediction = new_prediction;
            } else {
                break;
            }
        }

        let pos = prediction.get_block_pos();

        blocks.push((pos, self.get_platform().0)); // TODO: Improve

        self.generate_platforms(blocks, size, floor_level, pos)
    }
}

struct CaveGenerator {
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

    fn get_block(&self) -> Block {
        let block = if self.collection.uniform {
            self.collection.blocks[self.index].clone()
        } else {
            self.collection.blocks.get_random().unwrap().clone()
        };
        block.into_block()
    }

    pub fn generate(&self) -> (BlockPos, Vec<(BlockPos, Block)>, BlockPos) {
        let mut blocks = Vec::new();

        let mut rng = rand::thread_rng();

        let mut size: IVec3 = IVec3::new(
            rng.gen_range(10..=20),
            rng.gen_range(8..=16),
            rng.gen_range(15..=60),
        );

        let start = BlockPos::new(size.x / 2, 2, 0);

        let mut platform_blocks = vec![(start, self.get_block())];
        let mut air = Vec::new();

        let end = self.generate_platforms(&mut air, &mut platform_blocks, &size, start);

        size.z = end.z + 1;

        self.fill(&mut blocks, &size);

        blocks.append(&mut air);
        blocks.append(&mut platform_blocks);

        (start, blocks, end)
    }

    fn fill(&self, blocks: &mut Vec<(BlockPos, Block)>, size: &IVec3) {
        let pos = BlockPos::new(0, 0, 0);

        for x in 0..size.x {
            for y in 0..size.y {
                for z in 0..size.z {
                    let pos = BlockPos::new(pos.x + x, pos.y + y, pos.z + z);
                    blocks.push((pos, self.get_block()));
                }
            }
        }
    }

    fn generate_platforms(
        &self,
        air: &mut Vec<(BlockPos, Block)>,
        blocks: &mut Vec<(BlockPos, Block)>,
        size: &IVec3,
        prev: BlockPos,
    ) -> BlockPos {
        if prev.z >= size.z - 1 {
            return prev;
        }

        let mut rng = rand::thread_rng();

        let (min_yaw, max_yaw) = get_min_max_yaw(prev, size);

        let yaw = -rng.gen_range(min_yaw..=max_yaw);

        let mut prediction = PredictionState::running_jump_block(prev, yaw);

        let target_y = if prev.y <= 1 {
            prev.y + 1
        } else if prev.y >= size.y - 3 {
            prev.y - rng.gen_range(1..=3)
        } else {
            prev.y + rng.gen_range(-1..=1)
        };

        let mut intersected_blocks = HashSet::new();

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

                return self.generate_platforms(air, blocks, size, prev);
            }

            if new_prediction.vel.y > 0. || new_prediction.pos.y > target_y as f64 {
                prediction = new_prediction;
                intersected_blocks.insert(prediction.get_intersected_blocks());
            } else {
                break;
            }
        }

        for b in intersected_blocks {
            for b in b {
                air.push((b, BlockState::AIR.into_block()));

                for y in 0..b.y {
                    air.push((BlockPos::new(b.x, y, b.z), BlockState::AIR.into_block()));
                }
            }
        }

        blocks.push((prediction.get_block_pos(), self.get_block()));

        self.generate_platforms(air, blocks, size, prediction.get_block_pos())
    }
}
