// I'm trying to somewhat copy TrackMania's grid system.

use std::collections::{HashMap, HashSet};

use rand::{seq::SliceRandom, Rng};
use valence::prelude::*;

use crate::{
    generation::generator::{BlockGenParams, BlockGenerator, GenerateResult},
    utils::ToBlockPos,
};

use super::custom_generation::get_block;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Top,
    Bottom,
    Left,
    Right,
}

impl Direction {
    pub fn get_opposite(&self) -> Direction {
        match self {
            Direction::Top => Direction::Bottom,
            Direction::Bottom => Direction::Top,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn get_left(&self) -> Direction {
        match self {
            Direction::Top => Direction::Left,
            Direction::Bottom => Direction::Right,
            Direction::Left => Direction::Bottom,
            Direction::Right => Direction::Top,
        }
    }

    pub fn get_right(&self) -> Direction {
        match self {
            Direction::Top => Direction::Right,
            Direction::Bottom => Direction::Left,
            Direction::Left => Direction::Top,
            Direction::Right => Direction::Bottom,
        }
    }

    pub fn get_orthogonal(&self) -> [Direction; 2] {
        [self.get_left(), self.get_right()]
    }

    pub fn get_forward_and_orthogonal(&self) -> [Direction; 3] {
        [self.clone(), self.get_left(), self.get_right()]
    }
}

impl ToBlockPos for Direction {
    fn to_block_pos(&self) -> BlockPos {
        match self {
            Direction::Top => BlockPos::new(0, 0, 1),
            Direction::Bottom => BlockPos::new(0, 0, -1),
            Direction::Left => BlockPos::new(-1, 0, 0),
            Direction::Right => BlockPos::new(1, 0, 0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComplexCell {
    pub connection_top: Option<(String, Direction)>,
    pub connection_bottom: Option<(String, Direction)>,
    pub connection_left: Option<(String, Direction)>,
    pub connection_right: Option<(String, Direction)>,
}

impl ComplexCell {
    pub fn get_next(&self, direction: Direction) -> Option<(String, Direction)> {
        match direction {
            Direction::Top => self.connection_top.clone(),
            Direction::Bottom => self.connection_bottom.clone(),
            Direction::Left => self.connection_left.clone(),
            Direction::Right => self.connection_right.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ComplexGenerator {
    // TODO: Might want to use Rc<ComplexCell> instead of cloning.
    pub cells: Vec<ComplexCell>,
    pub cells_by_top: HashMap<String, Vec<ComplexCell>>,
    pub cells_by_bottom: HashMap<String, Vec<ComplexCell>>,
    pub cells_by_left: HashMap<String, Vec<ComplexCell>>,
    pub cells_by_right: HashMap<String, Vec<ComplexCell>>,
    pub cell_grid: HashMap<BlockPos, ComplexCell>,
}

impl ComplexGenerator {
    pub fn new(cells: Vec<ComplexCell>) -> ComplexGenerator {
        let mut cells_by_top = HashMap::new();
        let mut cells_by_bottom = HashMap::new();
        let mut cells_by_left = HashMap::new();
        let mut cells_by_right = HashMap::new();
        let mut cell_grid = HashMap::new();
        for cell in &cells {
            if let Some((name, _)) = &cell.connection_top {
                cells_by_top
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(cell.clone());
            }
            if let Some((name, _)) = &cell.connection_bottom {
                cells_by_bottom
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(cell.clone());
            }
            if let Some((name, _)) = &cell.connection_left {
                cells_by_left
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(cell.clone());
            }
            if let Some((name, _)) = &cell.connection_right {
                cells_by_right
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(cell.clone());
            }
        }
        Self {
            cells,
            cells_by_top,
            cells_by_bottom,
            cells_by_left,
            cells_by_right,
            cell_grid,
        }
    }

    pub fn has_cell(&self, pos: BlockPos) -> bool {
        self.cell_grid.contains_key(&pos)
    }

    pub fn get_cell(&self, pos: BlockPos) -> Option<ComplexCell> {
        self.cell_grid.get(&pos).map(|c| c.clone())
    }

    pub fn get_cells_by_dir_name(
        &self,
        direction: Direction,
        name: &Option<String>,
    ) -> Option<Vec<ComplexCell>> {
        if let Some(name) = name {
            if let Some(v) = match direction {
                Direction::Top => self.cells_by_top.get(name),
                Direction::Bottom => self.cells_by_bottom.get(name),
                Direction::Left => self.cells_by_left.get(name),
                Direction::Right => self.cells_by_right.get(name),
            } {
                Some(v.clone())
            } else {
                None
            }
        } else {
            // just return all cells in that direction
            let vec: Vec<ComplexCell> = match direction {
                Direction::Top => &self.cells_by_top,
                Direction::Bottom => &self.cells_by_bottom,
                Direction::Left => &self.cells_by_left,
                Direction::Right => &self.cells_by_right,
            }
            .values()
            .flatten()
            .cloned()
            .collect();

            if vec.is_empty() {
                None
            } else {
                Some(vec)
            }
        }
    }

    /// Returns the end of the path including the direction and the name of the
    /// previous cell. The pos will always be empty. If the path loops back to
    /// the start, None is returned.
    fn get_end_of_path(
        &self,
        pos: BlockPos,
        direction: Direction,
    ) -> Option<(BlockPos, Direction, String)> {
        let mut current_pos = pos;
        let mut current_direction = direction;
        let mut current_name = String::new();
        loop {
            if let Some(cell) = self.get_cell(current_pos) {
                if let Some((_, next_direction)) = cell.get_next(current_direction.get_opposite()) {
                    current_pos = current_pos + next_direction.to_block_pos();
                    current_direction = next_direction;
                    current_name = cell
                        .get_next(current_direction)
                        .expect("Should have a name")
                        .0;

                    if current_direction == direction && current_pos == pos {
                        return None;
                    }
                    continue;
                }
            }
            return Some((current_pos, current_direction, current_name));
        }
    }

    pub fn get_placement(&self, pos: BlockPos, direction: Direction) -> PlacementResult {
        let cell = match self.get_cell(pos) {
            Some(c) => c,
            None => return PlacementResult::Invalid,
        };
        if let Some((next_name, next_direction)) = cell.get_next(direction.get_opposite()) {
            // Check to make sure that the next cell connects to the current cell
            if let Some(next_cell) = self.get_cell(pos + next_direction.to_block_pos()) {
                let next_cell_next = next_cell.get_next(next_direction.get_opposite());
                if let Some((next_cell_name, _)) = next_cell_next {
                    if next_cell_name != next_name {
                        return PlacementResult::Invalid;
                    }
                } else {
                    return PlacementResult::Invalid;
                }
            }

            // Make sure we're not making an infinite loop
            for direction in direction.get_forward_and_orthogonal() {
                if let None = self.get_end_of_path(pos, direction) {
                    return PlacementResult::Invalid;
                }
            }

            // We're good!

            // TODO: Instead of returning `Moved` or `Valid`, we should return
            // a list of blocks that we're allowed to place.
            if let Some((end_pos, end_direction, end_name)) = self.get_end_of_path(pos, direction) {
                // Connected to something. Follow the path.
                return PlacementResult::Moved(end_pos, end_direction, end_name);
            } else {
                // Not connected to anything. This is fine.
                return PlacementResult::Valid;
            }
        } else {
            return PlacementResult::Invalid;
        }
    }

    /// The depth-first search algorithm used to generate the path.
    /// Stops when it reaches the end of the grid (max.z).
    fn dfs(
        &mut self,
        min: BlockPos,
        max: BlockPos,
        current_pos: BlockPos, // doesn't exist in the grid
        current_direction: Direction,
        current_name: Option<String>,
        visited: &mut HashSet<BlockPos>,
    ) -> Option<BlockPos> {
        let mut rng = rand::thread_rng();
        let options = self.get_cells_by_dir_name(current_direction.get_opposite(), &current_name);
        if let Some(options) = options {
            let mut options = options.clone();
            options.shuffle(&mut rng);
            for cell in options {
                let (_, direction) = cell
                    .get_next(current_direction.get_opposite())
                    .expect("Cell should have been filtered out if it doesn't have a connection");
                let (name, _) = cell
                    .get_next(direction)
                    .expect("If the cell has a connection, it should have a name");
                let pos = current_pos + direction.to_block_pos();
                if pos.x < min.x
                    || pos.y < min.y
                    || pos.z < min.z
                    || pos.x > max.x
                    || pos.y > max.y
                    || pos.z > max.z
                {
                    continue;
                }
                if visited.contains(&pos) {
                    continue;
                }
                visited.insert(pos);

                if pos.z == max.z {
                    // We're done!
                    self.cell_grid.insert(current_pos, cell);
                    return Some(current_pos);
                }

                self.cell_grid.insert(current_pos, cell);
                match self.get_placement(current_pos, current_direction) {
                    PlacementResult::Moved(new_pos, new_direction, new_name) => {
                        if let Some(t) =
                            self.dfs(min, max, new_pos, new_direction, Some(new_name), visited)
                        {
                            return Some(t);
                        }
                    }
                    PlacementResult::Valid => {
                        if let Some(t) = self.dfs(min, max, pos, direction, Some(name), visited) {
                            return Some(t);
                        }
                    }
                    PlacementResult::Invalid => {}
                }
                self.cell_grid.remove(&current_pos);
            }
        }
        None
    }

    pub fn generate_dfs(&mut self, min: BlockPos, max: BlockPos) -> Option<BlockPos> {
        let mut visited = HashSet::new();
        let current_pos = BlockPos::new(0, 0, 0);
        let current_direction = Direction::Top;
        let current_name = None;

        return self.dfs(
            min,
            max,
            current_pos,
            current_direction,
            current_name.clone(),
            &mut visited,
        );
    }
}

impl BlockGenerator for ComplexGenerator {
    fn generate(&self, params: &BlockGenParams) -> GenerateResult {
        let mut blocks = HashMap::new();

        for (pos, cell) in &self.cell_grid {
            let pos = BlockPos::new(pos.x * 3, pos.y * 3, pos.z * 3);

            if let Some((name, _)) = &cell.connection_bottom {
                blocks.insert(
                    pos,
                    get_block(&(name.to_owned(), vec![]), &params.block_map),
                );
            }

            if let Some((name, _)) = &cell.connection_top {
                blocks.insert(
                    pos + BlockPos::new(0, 0, 2),
                    get_block(&(name.to_owned(), vec![]), &params.block_map),
                );
            }

            if let Some((name, _)) = &cell.connection_left {
                blocks.insert(
                    pos + BlockPos::new(-1, 0, 1),
                    get_block(&(name.to_owned(), vec![]), &params.block_map),
                );
            }

            if let Some((name, _)) = &cell.connection_right {
                blocks.insert(
                    pos + BlockPos::new(1, 0, 1),
                    get_block(&(name.to_owned(), vec![]), &params.block_map),
                );
            }

            blocks.insert(pos + BlockPos::new(0, 0, 1), BlockState::STONE);
        }

        GenerateResult::just_blocks(blocks, BlockPos::new(0, 0, 0), BlockPos::new(0, 0, 0))
    }
}

#[derive(Clone, Debug)]
pub enum PlacementResult {
    Moved(BlockPos, Direction, String),
    Valid,
    Invalid,
}
