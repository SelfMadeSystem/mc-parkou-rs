// I'm trying to somewhat copy TrackMania's grid system.

use std::collections::{HashMap, HashSet};

use rand::seq::SliceRandom;
use valence::prelude::*;

use crate::{
    generation::{
        block_collection::BuiltBlockCollectionMap,
        block_grid::BlockGrid,
        generator::{BlockGenParams, BlockGenerator, GenerateResult},
    },
    utils::*,
};

/// I require to create my own `Direction` instead of using `valence::Direction`
/// because `valence::Direction` doesn't implement `Hash`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Direction {
    #[default]
    North,
    South,
    West,
    East,
    Up,
    Down,
}

impl Direction {
    pub fn get_opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    pub fn get_left(&self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
            Direction::East => Direction::North,
            Direction::Up => Direction::Up,
            Direction::Down => Direction::Down,
        }
    }

    pub fn get_right(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
            Direction::East => Direction::South,
            Direction::Up => Direction::Up,
            Direction::Down => Direction::Down,
        }
    }

    pub fn mirror_horizontal(&self) -> Direction {
        match self {
            Direction::North => Direction::North,
            Direction::South => Direction::South,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
            Direction::Up => Direction::Up,
            Direction::Down => Direction::Down,
        }
    }

    pub fn get_forward_and_orthogonal(&self) -> [Direction; 3] {
        [self.clone(), self.get_left(), self.get_right()]
    }
}

impl ToBlockPos for Direction {
    fn to_block_pos(&self) -> BlockPos {
        match self {
            Direction::North => BlockPos::new(0, 0, -1),
            Direction::South => BlockPos::new(0, 0, 1),
            Direction::West => BlockPos::new(-1, 0, 0),
            Direction::East => BlockPos::new(1, 0, 0),
            Direction::Up => BlockPos::new(0, 1, 0),
            Direction::Down => BlockPos::new(0, -1, 0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Connection {
    pub name: String,
    pub next_direction: Direction,
    /// If true, this tile can be placed using this connection
    pub can_next: bool,
    /// If true, this tile can be the start of the path
    pub can_start: bool,
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            name: Default::default(),
            next_direction: Default::default(),
            can_next: true,
            can_start: true,
        }
    }
}

impl Connection {
    pub fn rotate_cw(&self) -> Connection {
        Connection {
            next_direction: self.next_direction.get_right(),
            ..self.clone()
        }
    }

    pub fn flip_x(&self) -> Connection {
        Connection {
            next_direction: self.next_direction.mirror_horizontal(),
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ComplexTile {
    pub connection_north: Option<Connection>,
    pub connection_south: Option<Connection>,
    pub connection_west: Option<Connection>,
    pub connection_east: Option<Connection>,
    pub connection_up: Option<Connection>,
    pub connection_down: Option<Connection>,
    pub grid: BlockGrid,    // ignore this field in Hash and PartialEq
    pub disable_flip: bool, // TODO: Figure out if I should add `disable_rotate` too
}

impl Eq for ComplexTile {}

impl std::hash::Hash for ComplexTile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.connection_north.hash(state);
        self.connection_south.hash(state);
        self.connection_west.hash(state);
        self.connection_east.hash(state);
        self.connection_up.hash(state);
        self.connection_down.hash(state);
        self.disable_flip.hash(state);
    }
}

impl PartialEq for ComplexTile {
    fn eq(&self, other: &Self) -> bool {
        self.connection_north == other.connection_north
            && self.connection_south == other.connection_south
            && self.connection_west == other.connection_west
            && self.connection_east == other.connection_east
            && self.connection_up == other.connection_up
            && self.connection_down == other.connection_down
            && self.disable_flip == other.disable_flip
    }
}

impl ComplexTile {
    pub fn get_next(&self, direction: Direction) -> Option<Connection> {
        match direction {
            Direction::North => self.connection_north.clone(),
            Direction::South => self.connection_south.clone(),
            Direction::West => self.connection_west.clone(),
            Direction::East => self.connection_east.clone(),
            Direction::Up => self.connection_up.clone(),
            Direction::Down => self.connection_down.clone(),
        }
    }

    /// Returns the tile rotated 90 degrees clockwise
    pub fn rotate_cw(&self, origin: BlockPos) -> ComplexTile {
        ComplexTile {
            connection_north: self.connection_west.as_ref().map(|c| c.rotate_cw()),
            connection_south: self.connection_east.as_ref().map(|c| c.rotate_cw()),
            connection_west: self.connection_south.as_ref().map(|c| c.rotate_cw()),
            connection_east: self.connection_north.as_ref().map(|c| c.rotate_cw()),
            connection_up: self.connection_up.as_ref().map(|c| c.rotate_cw()),
            connection_down: self.connection_down.as_ref().map(|c| c.rotate_cw()),
            grid: self.grid.rotate_cw(origin),
            ..self.clone()
        }
    }

    /// Returns the tile flipped along the X axis
    pub fn flip_x(&self, origin: BlockPos) -> ComplexTile {
        ComplexTile {
            connection_north: self.connection_north.as_ref().map(|c| c.flip_x()),
            connection_south: self.connection_south.as_ref().map(|c| c.flip_x()),
            connection_west: self.connection_east.as_ref().map(|c| c.flip_x()),
            connection_east: self.connection_west.as_ref().map(|c| c.flip_x()),
            connection_up: self.connection_up.as_ref().map(|c| c.flip_x()),
            connection_down: self.connection_down.as_ref().map(|c| c.flip_x()),
            grid: self.grid.flip_x(origin),
            ..self.clone()
        }
    }

    /// Returns all the rotated and mirrored versions of the tile, without duplicates
    pub fn get_all_rotations(&self, origin: BlockPos) -> Vec<ComplexTile> {
        let mut tiles = HashSet::new();
        let mut current_tile = self.clone();
        for _ in 0..4 {
            tiles.insert(current_tile.clone());
            if !self.disable_flip {
                tiles.insert(current_tile.flip_x(origin));
            }
            current_tile = current_tile.rotate_cw(origin);
        }
        tiles.into_iter().collect()
    }

    /// Places the tile in the grid at the given position
    fn place(
        &self,
        grid: &mut HashMap<BlockPos, BlockState>,
        block_map: &BuiltBlockCollectionMap,
        pos: BlockPos,
    ) {
        for (block_pos, block) in &self.grid.blocks {
            let block_pos = pos + *block_pos;
            let block = block.get_block(&block_map);
            grid.insert(block_pos, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct ComplexGenerator {
    // TODO: Might want to use Rc<ComplexTile> instead of cloning.
    pub tile_size: BlockPos,
    pub min_pos: BlockPos,
    pub max_pos: BlockPos,
    pub tiles: Vec<ComplexTile>,
    pub starting_tiles: Vec<ComplexTile>,
    pub tiles_by_north: HashMap<String, Vec<ComplexTile>>,
    pub tiles_by_south: HashMap<String, Vec<ComplexTile>>,
    pub tiles_by_west: HashMap<String, Vec<ComplexTile>>,
    pub tiles_by_east: HashMap<String, Vec<ComplexTile>>,
    pub tiles_by_up: HashMap<String, Vec<ComplexTile>>,
    pub tiles_by_down: HashMap<String, Vec<ComplexTile>>,
    pub tile_grid: HashMap<BlockPos, ComplexTile>,
}

impl ComplexGenerator {
    pub fn new(
        tiles: Vec<ComplexTile>,
        tile_size: BlockPos,
        min_pos: BlockPos,
        max_pos: BlockPos,
    ) -> ComplexGenerator {
        let mut new_tiles = Vec::new();
        let origin = BlockPos::new(0, 0, tile_size.z / 2);
        for tile in tiles {
            new_tiles.extend(tile.get_all_rotations(origin));
        }

        let tiles = new_tiles;
        let mut starting_tiles = Vec::new();
        let mut tiles_by_north = HashMap::new();
        let mut tiles_by_south = HashMap::new();
        let mut tiles_by_west = HashMap::new();
        let mut tiles_by_east = HashMap::new();
        let mut tiles_by_up = HashMap::new();
        let mut tiles_by_down = HashMap::new();

        let tile_grid = HashMap::new();

        for tile in &tiles {
            if let Some(Connection {
                name,
                can_next,
                can_start,
                ..
            }) = &tile.connection_north
            {
                if *can_next {
                    tiles_by_north
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(tile.clone());

                    if *can_start {
                        starting_tiles.push(tile.clone());
                    }
                }
            }
            if let Some(Connection { name, can_next, .. }) = &tile.connection_south {
                if *can_next {
                    tiles_by_south
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(tile.clone());
                }
            }
            if let Some(Connection { name, can_next, .. }) = &tile.connection_west {
                if *can_next {
                    tiles_by_west
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(tile.clone());
                }
            }
            if let Some(Connection { name, can_next, .. }) = &tile.connection_east {
                if *can_next {
                    tiles_by_east
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(tile.clone());
                }
            }
            if let Some(Connection { name, can_next, .. }) = &tile.connection_up {
                if *can_next {
                    tiles_by_up
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(tile.clone());
                }
            }
            if let Some(Connection { name, can_next, .. }) = &tile.connection_down {
                if *can_next {
                    tiles_by_down
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(tile.clone());
                }
            }
        }
        Self {
            tile_size,
            min_pos,
            max_pos,
            tiles,
            starting_tiles,
            tiles_by_north,
            tiles_by_south,
            tiles_by_west,
            tiles_by_east,
            tiles_by_up,
            tiles_by_down,
            tile_grid,
        }
    }

    pub fn get_tile(&self, pos: BlockPos) -> Option<&ComplexTile> {
        self.tile_grid.get(&pos)
    }

    pub fn get_tiles_by_dir_name(
        &self,
        direction: Direction,
        name: &str,
    ) -> Option<Vec<ComplexTile>> {
        if let Some(v) = match direction {
            Direction::North => self.tiles_by_north.get(name),
            Direction::South => self.tiles_by_south.get(name),
            Direction::West => self.tiles_by_west.get(name),
            Direction::East => self.tiles_by_east.get(name),
            Direction::Up => self.tiles_by_up.get(name),
            Direction::Down => self.tiles_by_down.get(name),
        } {
            Some(v.clone())
        } else {
            None
        }
    }

    /// Returns the end of the path including the direction and the name of the
    /// previous tile. The pos will always be empty. If the path loops back to
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
            if let Some(tile) = self.get_tile(current_pos) {
                if let Some(Connection { next_direction, .. }) =
                    tile.get_next(current_direction.get_opposite())
                {
                    current_pos = current_pos + next_direction.to_block_pos();
                    current_direction = next_direction;
                    current_name = tile
                        .get_next(current_direction)
                        .expect("Should have a name")
                        .name;

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
    ) -> Option<(BlockPos, Direction, Vec<ComplexTile>)> {
        let tile = match self.get_tile(pos) {
            Some(c) => c,
            None => return None,
        };
        if let Some(Connection {
            name: next_name,
            next_direction,
            ..
        }) = tile.get_next(direction.get_opposite())
        {
            // Check to make sure that the next tile connects to the current tile
            if let Some(next_tile) = self.get_tile(pos + next_direction.to_block_pos()) {
                let next_tile_next = next_tile.get_next(next_direction.get_opposite());
                if let Some(Connection {
                    name: next_tile_name,
                    ..
                }) = next_tile_next
                {
                    if next_tile_name != next_name {
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

            // Get the possible tiles that can be placed here
            let mut tiles = self.get_tiles_by_dir_name(direction.get_opposite(), &name)?;

            // Filter out tiles that don't connect to the adjacent tiles
            tiles = tiles
                .into_iter()
                .filter(|tile| {
                    for direction in direction.get_forward_and_orthogonal() {
                        if let Some(next_tile) = self.get_tile(pos + direction.to_block_pos()) {
                            let our_next = tile.get_next(direction);
                            let their_next = next_tile.get_next(direction.get_opposite());

                            match (our_next, their_next) {
                                (
                                    Some(Connection { name: our_name, .. }),
                                    Some(Connection {
                                        name: their_name, ..
                                    }),
                                ) => {
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

            // There should be at least one tile left
            if tiles.is_empty() {
                return None;
            }

            // We're done!
            return Some((pos, direction, tiles));
        } else {
            return None;
        }
    }

    /// The depth-first search algorithm used to generate the path.
    /// Stops when it reaches the end of the grid (max.z).
    fn dfs(
        &mut self,
        current_pos: BlockPos, // doesn't exist in the grid
        current_direction: Direction,
        mut current_tiles: Vec<ComplexTile>,
        visited: &mut HashSet<BlockPos>,
    ) -> Option<BlockPos> {
        let mut rng = rand::thread_rng();
        current_tiles.shuffle(&mut rng);
        for tile in current_tiles {
            let Connection {
                next_direction: direction,
                ..
            } = tile
                .get_next(current_direction.get_opposite())
                .expect("Tile should have been filtered out if it doesn't have a connection");
            let Connection { name, .. } = tile
                .get_next(direction)
                .expect("If the tile has a connection, it should have a name");
            let pos = current_pos + direction.to_block_pos();
            if pos.x <     self.min_pos.x
                || pos.y < self.min_pos.y
                || pos.z < self.min_pos.z
                || pos.x > self.max_pos.x
                || pos.y > self.max_pos.y
                || pos.z > self.max_pos.z
            {
                continue;
            }

            if visited.contains(&pos) {
                continue;
            }

            if pos.z == self.max_pos.z {
                // We're done!
                self.tile_grid.insert(current_pos, tile);
                return Some(current_pos);
            }

            self.tile_grid.insert(current_pos, tile);
            match self.get_placement(current_pos, current_direction, name) {
                Some((new_pos, new_direction, new_tiles)) => {
                    if let Some(t) = self.dfs(new_pos, new_direction, new_tiles, visited)
                    {
                        return Some(t);
                    }
                }
                None => {}
            }
            visited.insert(pos);
            self.tile_grid.remove(&current_pos);
        }
        None
    }

    pub fn generate_dfs(&mut self) -> Option<BlockPos> {
        let mut visited = HashSet::new();
        let current_pos = BlockPos::new(0, 0, 0);
        let current_direction = Direction::South;
        let current_tiles = self.starting_tiles.clone();

        return self.dfs(
            current_pos,
            current_direction,
            current_tiles,
            &mut visited,
        );
    }
}

impl BlockGenerator for ComplexGenerator {
    fn generate(&self, params: &BlockGenParams) -> GenerateResult {
        let mut blocks = HashMap::new();

        for (pos, tile) in &self.tile_grid {
            let pos = (pos.to_vec3() * self.tile_size.to_vec3()).to_block_pos();

            tile.place(&mut blocks, &params.block_map, pos);
        }

        GenerateResult::just_blocks(blocks, BlockPos::new(0, 0, 0), BlockPos::new(0, 0, 0))
    }
}
