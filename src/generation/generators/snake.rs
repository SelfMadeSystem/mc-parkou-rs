use std::collections::{HashMap, HashSet};

use rand::{seq::SliceRandom, Rng};
use valence::{math::*, prelude::*};

use crate::{
    alt_block::{AltBlock, AltBlockState},
    generation::{block_collection::*, generator::GenerateResult, generation::ChildGeneration},
    utils::*,
};

pub struct SnakeGenerator {
    pub blocks: BlockChoice<BlockState>,
    pub snake_count: usize,
    pub snake_length: usize,
    pub delay: usize,
    pub reverse: bool,
    pub poses: Vec<BlockPos>,
    pub end_pos: BlockPos,
    pub index: usize,
}

impl SnakeGenerator {
    pub fn new(
        blocks: BlockChoice<BlockState>,
        snake_count: usize,
        snake_length: usize,
        delay: usize,
        reverse: bool,
    ) -> Self {
        let i = blocks.blocks.get_random_index().unwrap();
        Self {
            blocks,
            snake_count,
            snake_length,
            delay,
            reverse,
            poses: Vec::new(),
            end_pos: BlockPos::new(0, 0, 0),
            index: i,
        }
    }

    fn get_block(&self) -> BlockState {
        if self.blocks.uniform {
            self.blocks.blocks[self.index].clone()
        } else {
            self.blocks.blocks.get_random().unwrap().clone()
        }
    }

    pub fn add_block(&mut self, pos: BlockPos) {
        self.poses.push(pos);
    }

    /// Sets the end position to the last position in the snake.
    /// If the snake is empty, the end position is not set.
    pub fn set_end(&mut self) {
        let opt_last = self.poses.last();
        if let Some(last) = opt_last {
            self.end_pos = *last;
        }
    }

    pub fn add_direction(&mut self, dir: BlockPos) {
        let opt_last = self.poses.last();
        let last;
        if let Some(l) = opt_last {
            last = *l;
        } else {
            self.poses.push(BlockPos::new(0, 0, 0));
            return;
        }
        let pos = last + dir;
        self.poses.push(pos);
    }

    pub fn can_go(&self, dir: BlockPos) -> bool {
        let opt_last = self.poses.last();
        let last;
        if let Some(l) = opt_last {
            last = *l;
        } else {
            return true;
        }
        let pos1 = last + dir;
        let pos2 = pos1 + dir;
        !self.poses.contains(&pos1) && !self.poses.contains(&pos2)
    }

    /// Creates a 2D snake that loops back on itself.
    /// Uses a backtracking depth-first search algorithm.
    pub fn create_looping_snake(&mut self, min: BlockPos, max: BlockPos) {
        self.poses.clear();
        while !self.dfs_looping(
            min,
            max,
            BlockPos::new(0, 0, 0),
            BlockPos::new(0, 0, 0),
            &mut HashSet::from([BlockPos::new(0, 0, 0)]),
            0,
        ) {}
        self.set_end_random();
    }

    fn dfs_looping(
        &mut self,
        min: BlockPos,
        max: BlockPos,
        current: BlockPos,
        prev: BlockPos,
        visited: &mut HashSet<BlockPos>,
        mut down: isize,
    ) -> bool {
        // TODO: Figure out if this is the best way to do this
        // It sometimes decides to go in an infinite loop
        let mut directions = vec![
            BlockPos::new(1, 0, 0),
            BlockPos::new(-1, 0, 0),
            BlockPos::new(0, 0, 1),
            BlockPos::new(0, 0, -1),
        ];

        let mut rng = rand::thread_rng();

        let mut i_decided_to_go_down = false;

        if down == 0 {
            if rng.gen_bool(0.1) {
                i_decided_to_go_down = true;
                down = rng.gen_range(3..=5) + 1;
            }
        }

        directions.shuffle(&mut rng);

        for dir in directions {
            let mut pos = current + dir;
            if pos.x < min.x
                || pos.x > max.x
                || pos.y < min.y
                || pos.y > max.y
                || pos.z < min.z
                || pos.z > max.z
            {
                continue;
            }

            if pos + dir == BlockPos::new(0, 0, 0) {
                if down > 0 && !i_decided_to_go_down {
                    self.poses.push(pos + dir);
                    pos.y -= 1;
                    self.poses.push(pos + dir);
                    pos.y -= 1;
                    self.poses.push(pos + dir);
                    self.poses.push(pos);
                } else {
                    self.poses.push(pos + dir);
                    self.poses.push(pos);
                }
                return true;
            }

            if pos == prev {
                continue;
            }

            if visited.contains(&pos)
                || visited.contains(&(pos + dir))
                || get_dirs_next_to(dir)
                    .iter()
                    .any(|d| visited.contains(&(pos + *d)))
            {
                continue;
            }

            visited.insert(pos);

            if self.dfs_looping(min, max, pos, current, visited, (down - 1).max(0)) {
                if i_decided_to_go_down {
                    pos.y -= 2;
                    self.poses.push(pos);
                    pos.y += 1;
                    self.poses.push(pos);
                    pos.y += 1;
                } else if down == 1 {
                    self.poses.push(pos);
                    pos.y -= 1;
                    self.poses.push(pos);
                    pos.y -= 1;
                } else if down > 1 {
                    pos.y -= 2;
                }
                self.poses.push(pos);
                return true;
            }
        }

        false
    }

    /// Sets the end by picking a random position furthest in the Z direction.
    pub fn set_end_random(&mut self) {
        let mut rng = rand::thread_rng();
        let mut max = 0;
        let mut max_poses = Vec::new();
        for pos in &self.poses {
            if pos.z > max {
                max = pos.z;
                max_poses.clear();
            }

            if pos.z == max {
                max_poses.push(*pos);
            }
        }
        self.end_pos = *max_poses.choose(&mut rng).unwrap();
    }

    /// Gets children by finding the positions that are at the top of the snake
    /// and grouping together the positions that are next to each other.
    pub fn acquire_children(&self) -> Vec<ChildGeneration> {
        let mut children = Vec::new();
        let mut start_vec = Vec::new();
        let mut current_vec = Vec::new();

        let mut at_start = true;

        for pos in &self.poses {
            if pos.y == 0 {
                if at_start {
                    start_vec.push(*pos);
                } else {
                    current_vec.push(*pos);
                }
            } else {
                if at_start {
                    at_start = false;
                } else {
                    children.push(current_vec);
                    current_vec = Vec::new();
                }
            }
        }

        if !current_vec.is_empty() {
            start_vec.append(&mut current_vec);
        }

        if !start_vec.is_empty() {
            children.push(start_vec);
        }

        children.into_iter().map(|c| ChildGeneration::new(
            c.into_iter().map(|p| (p, self.get_block())).collect(),
            HashMap::new(),
        )).collect()
    }

    pub fn generate(&self) -> GenerateResult {
        let mut blocks = HashMap::new();
        let mut alt_blocks = HashMap::new();

        let total_blocks = self.poses.len() / self.snake_count;

        for (mut i, pos) in self.poses.iter().enumerate() {
            if self.reverse {
                i = self.poses.len() - i - 1;
            }
            let block = self.get_block();
            blocks.insert(*pos, block);

            alt_blocks.insert(
                *pos,
                AltBlock::Tick(
                    vec![
                        (AltBlockState::Block(block), self.snake_length * self.delay),
                        (
                            AltBlockState::SmallBlock(block),
                            (total_blocks - self.snake_length) * self.delay,
                        ),
                    ],
                    i * self.delay,
                ),
            );
        }

        GenerateResult {
            start: BlockPos::new(0, 0, 0),
            end: self.end_pos,
            blocks,
            alt_blocks,
            lines: Vec::new(),
            children: self.acquire_children(),
        }
    }
}
