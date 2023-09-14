use std::collections::{HashMap, HashSet};

use crate::{line::Line3, prediction::prediction_state::PredictionState, utils::*};

use super::{block_collection::*, generation::*, theme::GenerationTheme};
use rand::Rng;
use valence::{
    math::{IVec2, IVec3},
    prelude::*,
};

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

    pub fn generate(
        &self,
        direction: JumpDirection,
        yaw: f32,
        mut lines: Vec<Line3>,
    ) -> Generation {
        let mut blocks = HashMap::new();
        let mut offset: BlockPos = self.start;
        let mut children = Vec::new();
        let end_state: PredictionState;

        match &self.generation_type {
            GenerationType::Single(BlockCollection(collection)) => {
                blocks.insert(
                    BlockPos::new(0, 0, 0),
                    *collection
                        .blocks
                        .get_random()
                        .expect("No blocks in block collection"),
                );
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

                let get_block = || get_block_slab().block;

                let get_slab = || {
                    let slab = get_block_slab().slab;
                    (slab, slab.set(PropName::Type, PropValue::Top))
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
                    blocks.insert(pos, block);
                }

                end_state = PredictionState::running_jump_block(
                    self.start + pos.round().as_block_pos(),
                    new_yaw,
                );
            }
            GenerationType::Indoor(collection) => {
                let indoor = IndoorGenerator::new(collection.clone());

                let (start, bloccs, end, linez, childrenz) = indoor.generate();

                offset = offset - start;
                blocks = bloccs;
                children = childrenz;
                end_state = PredictionState::running_jump_block(offset + end, random_yaw_dist(30.)); // walls can be in the way

                for line in linez {
                    lines.push(line + offset.to_vec3());
                }
            }
            GenerationType::Cave(BlockCollection(collection)) => {
                let cave = CaveGenerator::new(collection.clone());

                let (start, bloccs, end, linez, childrenz) = cave.generate();

                offset = offset - start;
                blocks = bloccs;
                children = childrenz;
                end_state = PredictionState::running_jump_block(offset + end, random_yaw_dist(30.));

                for line in linez {
                    lines.push(line + offset.to_vec3());
                }
            }
        }

        Generation::new(blocks, children, offset, end_state, lines)
    }
}

struct IndoorGenerator {
    // TODO: NestedGenerator or CompositeGenerator or something idk
    // TODO: Integrate with the combo system
    collection: IndoorBlockCollection,
    wall_index: usize,
    floor_index: usize,
    platform_index: usize,
}

type GenerateResult = (
    BlockPos,
    HashMap<BlockPos, BlockState>,
    BlockPos,
    Vec<Line3>,
    Vec<ChildGeneration>,
);

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

        (start, blocks, end, lines, children)
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

        children.push(ChildGeneration::new(HashMap::from([(
            pos,
            self.get_platform().0,
        )])));

        lines.append(&mut new_lines);

        self.generate_platforms(size, floor_level, pos, lines, children)
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
        // FIXME: If 2 down, sometimes the ramp back up gets cut off.
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

        children.push(ChildGeneration::new(blocks));

        self.generate_platforms(air, children, size, pos, xz_air, floor_level, lines)
    }
}
