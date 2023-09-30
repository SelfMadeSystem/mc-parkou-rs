// I'm trying to somewhat copy TrackMania's grid system.

use std::collections::{HashMap, HashSet};

use rand::seq::SliceRandom;
use valence::prelude::*;

use crate::{
    generation::generator::{BlockGenParams, BlockGenerator, GenerateResult},
    utils::ToBlockPos,
};

use super::custom_generation::get_block;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

    pub fn mirror_horizontal(&self) -> Direction {
        match self {
            Direction::Top => Direction::Top,
            Direction::Bottom => Direction::Bottom,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

    /// Returns the cell rotated 90 degrees clockwise
    pub fn rotate_cw(&self) -> ComplexCell {
        ComplexCell {
            connection_top: match &self.connection_left {
                Some((name, direction)) => Some((name.clone(), direction.get_right())),
                None => None,
            },
            connection_bottom: match &self.connection_right {
                Some((name, direction)) => Some((name.clone(), direction.get_right())),
                None => None,
            },
            connection_left: match &self.connection_bottom {
                Some((name, direction)) => Some((name.clone(), direction.get_right())),
                None => None,
            },
            connection_right: match &self.connection_top {
                Some((name, direction)) => Some((name.clone(), direction.get_right())),
                None => None,
            },
        }
    }

    /// Returns the cell mirrored horizontally
    pub fn mirror_horizontal(&self) -> ComplexCell {
        ComplexCell {
            connection_top: match &self.connection_top {
                Some((name, direction)) => Some((name.clone(), direction.mirror_horizontal())),
                None => None,
            },
            connection_bottom: match &self.connection_bottom {
                Some((name, direction)) => Some((name.clone(), direction.mirror_horizontal())),
                None => None,
            },
            connection_left: match &self.connection_right {
                Some((name, direction)) => Some((name.clone(), direction.mirror_horizontal())),
                None => None,
            },
            connection_right: match &self.connection_left {
                Some((name, direction)) => Some((name.clone(), direction.mirror_horizontal())),
                None => None,
            },
        }
    }

    /// Returns all the rotated and mirrored versions of the cell, without duplicates
    pub fn get_all_rotations(&self) -> Vec<ComplexCell> {
        let mut cells = HashSet::new();
        let mut current_cell = self.clone();
        for _ in 0..4 {
            cells.insert(current_cell.clone());
            cells.insert(current_cell.mirror_horizontal());
            current_cell = current_cell.rotate_cw();
        }
        cells.into_iter().collect()
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
        let mut new_cells = Vec::new();
        for cell in cells {
            new_cells.extend(cell.get_all_rotations());
        }

        let cells = new_cells;
        let mut cells_by_top = HashMap::new();
        let mut cells_by_bottom = HashMap::new();
        let mut cells_by_left = HashMap::new();
        let mut cells_by_right = HashMap::new();
        let cell_grid = HashMap::new();
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
        name: Option<&str>,
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
        name: &str,
    ) -> Option<(BlockPos, Direction, String)> {
        let mut current_pos = pos;
        let mut current_direction = direction;
        let mut current_name = name.to_owned();
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

    pub fn get_placement(
        &self,
        pos: BlockPos,
        direction: Direction,
        name: String,
    ) -> Option<(BlockPos, Direction, Vec<ComplexCell>)> {
        let cell = match self.get_cell(pos) {
            Some(c) => c,
            None => return None,
        };
        if let Some((next_name, next_direction)) = cell.get_next(direction.get_opposite()) {
            // Check to make sure that the next cell connects to the current cell
            if let Some(next_cell) = self.get_cell(pos + next_direction.to_block_pos()) {
                let next_cell_next = next_cell.get_next(next_direction.get_opposite());
                if let Some((next_cell_name, _)) = next_cell_next {
                    if next_cell_name != next_name {
                        return None;
                    }
                } else {
                    return None;
                }
            }

            // Make sure we're not making an infinite loop
            for direction in direction.get_forward_and_orthogonal() {
                if let None = self.get_end_of_path(pos, direction, &name) {
                    return None;
                }
            }

            // We're good!

            // Get current pos, direction, and name
            let (pos, direction, name) = self
                .get_end_of_path(pos, direction, &name)
                .expect("Should have a path");

            // Get the possible cells that can be placed here
            let mut cells = self.get_cells_by_dir_name(direction.get_opposite(), Some(&name))?;

            // Filter out cells that don't connect to the adjacent cells
            cells = cells
                .into_iter()
                .filter(|cell| {
                    for direction in direction.get_forward_and_orthogonal() {
                        if let Some(next_cell) = self.get_cell(pos + direction.to_block_pos()) {
                            let our_next = cell.get_next(direction);
                            let their_next = next_cell.get_next(direction.get_opposite());

                            match (our_next, their_next) {
                                (Some((our_name, _)), Some((their_name, _))) => {
                                    if our_name != their_name {
                                        return false;
                                    }
                                }
                                (None, None) => {}
                                _ => return false,
                            }
                        }
                    }
                    true
                })
                .collect();

            // There should be at least one cell left
            if cells.is_empty() {
                return None;
            }

            // We're done!
            return Some((pos, direction, cells));
        } else {
            return None;
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
        mut current_cells: Vec<ComplexCell>,
        visited: &mut HashSet<BlockPos>,
    ) -> Option<BlockPos> {
        let mut rng = rand::thread_rng();
        current_cells.shuffle(&mut rng);
        for cell in current_cells {
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
            if visited.contains(&pos) && !self.has_cell(pos) {
                continue;
            }
            visited.insert(pos);

            if pos.z == max.z {
                // We're done!
                self.cell_grid.insert(current_pos, cell);
                return Some(current_pos);
            }

            self.cell_grid.insert(current_pos, cell);
            match self.get_placement(current_pos, current_direction, name) {
                Some((new_pos, new_direction, new_cells)) => {
                    if let Some(t) = self.dfs(min, max, new_pos, new_direction, new_cells, visited)
                    {
                        return Some(t);
                    }
                }
                None => {}
            }
            self.cell_grid.remove(&current_pos);
        }
        None
    }

    pub fn generate_dfs(&mut self, min: BlockPos, max: BlockPos) -> Option<BlockPos> {
        let mut visited = HashSet::new();
        let current_pos = BlockPos::new(0, 0, 0);
        let current_direction = Direction::Top;
        let current_cells = self
            .get_cells_by_dir_name(current_direction.get_opposite(), None)
            .expect("There should be at least one cell");

        return self.dfs(
            min,
            max,
            current_pos,
            current_direction,
            current_cells,
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

#[cfg(test)]
mod tests {
    use valence::BlockPos;

    use crate::generation::generators::complex_gen::ComplexCell;

    use super::{ComplexGenerator, Direction};

    #[test]
    fn test_end_of_path_tt() {
        let mut generator = ComplexGenerator::new(vec![]); // doesn't matter

        generator.cell_grid.insert(
            BlockPos::new(0, 0, 0),
            ComplexCell {
                connection_top: Some(("a".to_owned(), Direction::Bottom)),
                connection_bottom: Some(("a".to_owned(), Direction::Top)),
                connection_left: None,
                connection_right: None,
            },
        );

        generator.cell_grid.insert(
            BlockPos::new(0, 0, 1),
            ComplexCell {
                connection_top: Some(("b".to_owned(), Direction::Bottom)),
                connection_bottom: Some(("a".to_owned(), Direction::Top)),
                connection_left: None,
                connection_right: None,
            },
        );

        let (pos, direction, name) = generator
            .get_end_of_path(BlockPos::new(0, 0, 0), Direction::Top, "a")
            .expect("Should have a path");

        assert_eq!(pos, BlockPos::new(0, 0, 2));
        assert_eq!(direction, Direction::Top);
        assert_eq!(name, "b");
    }

    #[test]
    fn test_end_of_path_lt() {
        let mut generator = ComplexGenerator::new(vec![]); // doesn't matter

        generator.cell_grid.insert(
            BlockPos::new(0, 0, 0),
            ComplexCell {
                connection_top: None,
                connection_bottom: Some(("a".to_owned(), Direction::Left)),
                connection_left: Some(("a".to_owned(), Direction::Bottom)),
                connection_right: None,
            },
        );

        generator.cell_grid.insert(
            BlockPos::new(-1, 0, 0),
            ComplexCell {
                connection_top: Some(("b".to_owned(), Direction::Right)),
                connection_bottom: None,
                connection_left: None,
                connection_right: Some(("a".to_owned(), Direction::Top)),
            },
        );

        let (pos, direction, name) = generator
            .get_end_of_path(BlockPos::new(0, 0, 0), Direction::Top, "a")
            .expect("Should have a path");

        assert_eq!(pos, BlockPos::new(-1, 0, 1));
        assert_eq!(direction, Direction::Top);
        assert_eq!(name, "b");
    }

    #[test]
    fn test_end_of_path_lltr() {
        let mut generator = ComplexGenerator::new(vec![]); // doesn't matter

        generator.cell_grid.insert(
            BlockPos::new(0, 0, 0),
            ComplexCell {
                connection_top: None,
                connection_bottom: Some(("a".to_owned(), Direction::Left)),
                connection_left: Some(("a".to_owned(), Direction::Bottom)),
                connection_right: None,
            },
        );

        generator.cell_grid.insert(
            BlockPos::new(-1, 0, 0),
            ComplexCell {
                connection_top: None,
                connection_bottom: None,
                connection_left: Some(("a".to_owned(), Direction::Right)),
                connection_right: Some(("a".to_owned(), Direction::Left)),
            },
        );

        generator.cell_grid.insert(
            BlockPos::new(-2, 0, 0),
            ComplexCell {
                connection_top: Some(("a".to_owned(), Direction::Right)),
                connection_bottom: None,
                connection_left: None,
                connection_right: Some(("a".to_owned(), Direction::Top)),
            },
        );

        generator.cell_grid.insert(
            BlockPos::new(-2, 0, 1),
            ComplexCell {
                connection_top: None,
                connection_bottom: Some(("a".to_owned(), Direction::Right)),
                connection_left: None,
                connection_right: Some(("b".to_owned(), Direction::Top)),
            },
        );

        let (pos, direction, name) = generator
            .get_end_of_path(BlockPos::new(0, 0, 0), Direction::Top, "a")
            .expect("Should have a path");

        assert_eq!(pos, BlockPos::new(-1, 0, 1));
        assert_eq!(direction, Direction::Right);
        assert_eq!(name, "b");
    }

    #[test]
    fn test_complex_cell_rotate() {
        let cell = ComplexCell {
            connection_top: Some(("a".to_owned(), Direction::Bottom)),
            connection_bottom: Some(("a".to_owned(), Direction::Top)),
            connection_left: Some(("b".to_owned(), Direction::Right)),
            connection_right: Some(("b".to_owned(), Direction::Left)),
        };

        let rotateds = cell.get_all_rotations();

        assert_eq!(rotateds.len(), 2);

        let cell = ComplexCell {
            connection_top: Some(("a".to_owned(), Direction::Left)),
            connection_bottom: Some(("b".to_owned(), Direction::Right)),
            connection_left: Some(("a".to_owned(), Direction::Top)),
            connection_right: Some(("b".to_owned(), Direction::Bottom)),
        };

        let rotateds = cell.get_all_rotations();

        println!("{:?}", rotateds);
    }

    // TODO: Add more tests
}
