// I'm trying to somewhat copy TrackMania's grid system.

use std::collections::{HashMap, HashSet};

use rand::seq::SliceRandom;
use valence::prelude::*;

use crate::{
    generation::{
        block_collection::BuiltBlockCollectionMap,
        block_grid::BlockGrid,
        generation::ChildGeneration,
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

    pub fn get_forward_and_orthogonal(&self) -> [Direction; 5] {
        match self {
            Direction::Up | Direction::Down => [
                self.clone(),
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ],
            _ => [
                self.clone(),
                self.get_left(),
                self.get_right(),
                Direction::Up,
                Direction::Down,
            ],
        }
    }

    pub fn is_news(&self) -> bool {
        match self {
            Direction::North | Direction::South | Direction::West | Direction::East => true,
            _ => false,
        }
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

#[derive(Clone, Debug, Eq)]
pub struct Connection {
    pub name: String,
    pub next_direction: Direction,
    /// If true, this tile can be placed using this connection
    pub can_next: bool,
    /// If true, this tile can be the start of the path
    pub can_start: bool,
    /// The blocks that are part of this connection. If None, then the next
    /// connection's blocks will be used and it is assumed to be continuous.
    pub blocks: Option<HashSet<BlockPos>>,
    /// Only used with Up and Down connections. If Some, then the orientation of
    /// the previous tile will be used to determine the orientation of this tile.
    /// Must be either North, South, West, or East and must be present in the
    /// if this connection is Up or Down.
    pub attach_direction: Option<Direction>,
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.next_direction == other.next_direction
            && self.can_next == other.can_next
            && self.can_start == other.can_start
            && self.blocks.as_ref().map_or(0, |a| a.len())
                == other.blocks.as_ref().map_or(0, |a| a.len()) // TODO: I might not need this
    }
}

impl std::hash::Hash for Connection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.next_direction.hash(state);
        self.can_next.hash(state);
        self.can_start.hash(state);
        self.blocks.as_ref().map_or(0, |a| a.len()).hash(state); // TODO: I might not need this
    }
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            name: Default::default(),
            next_direction: Default::default(),
            can_next: true,
            can_start: true,
            blocks: None,
            attach_direction: None,
        }
    }
}

impl Connection {
    pub fn rotate_cw(&self, origin: BlockPos) -> Connection {
        Connection {
            next_direction: self.next_direction.get_right(),
            blocks: self
                .blocks
                .as_ref()
                .map(|blocks| rotate_block_set_cw(blocks, origin)),
            attach_direction: self.attach_direction.as_ref().map(|d| d.get_right()),
            ..self.clone()
        }
    }

    pub fn flip_x(&self, origin: BlockPos) -> Connection {
        Connection {
            next_direction: self.next_direction.mirror_horizontal(),
            blocks: self
                .blocks
                .as_ref()
                .map(|blocks| flip_block_set_x(blocks, origin)),
            attach_direction: self
                .attach_direction
                .as_ref()
                .map(|d| d.mirror_horizontal()),
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
            connection_north: self.connection_west.as_ref().map(|c| c.rotate_cw(origin)),
            connection_south: self.connection_east.as_ref().map(|c| c.rotate_cw(origin)),
            connection_west: self.connection_south.as_ref().map(|c| c.rotate_cw(origin)),
            connection_east: self.connection_north.as_ref().map(|c| c.rotate_cw(origin)),
            connection_up: self.connection_up.as_ref().map(|c| c.rotate_cw(origin)),
            connection_down: self.connection_down.as_ref().map(|c| c.rotate_cw(origin)),
            grid: self.grid.rotate_cw(origin),
            ..self.clone()
        }
    }

    /// Returns the tile flipped along the X axis
    pub fn flip_x(&self, origin: BlockPos) -> ComplexTile {
        ComplexTile {
            connection_north: self.connection_north.as_ref().map(|c| c.flip_x(origin)),
            connection_south: self.connection_south.as_ref().map(|c| c.flip_x(origin)),
            connection_west: self.connection_east.as_ref().map(|c| c.flip_x(origin)),
            connection_east: self.connection_west.as_ref().map(|c| c.flip_x(origin)),
            connection_up: self.connection_up.as_ref().map(|c| c.flip_x(origin)),
            connection_down: self.connection_down.as_ref().map(|c| c.flip_x(origin)),
            grid: self.grid.flip_x(origin),
            ..self.clone()
        }
    }

    /// Returns all the rotated and mirrored versions of the tile, without duplicates
    pub fn get_all_rotations(&self, origin: BlockPos, square: bool) -> Vec<ComplexTile> {
        let mut tiles = HashSet::new();
        let mut current_tile = self.clone();
        for _ in 0..if square { 4 } else { 2 } {
            // if square, rotate 4 times, else rotate 2 times twice.
            tiles.insert(current_tile.clone());
            if !self.disable_flip {
                tiles.insert(current_tile.flip_x(origin));
            }
            current_tile = current_tile.rotate_cw(origin);
            if !square {
                current_tile = current_tile.rotate_cw(origin); // rotate 2 times for non-square grid
            }
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

    /// Verifies that the tile is valid.
    ///
    /// A tile is valid if it has at least two connections and every
    /// connection's `next_direction` points to a connection that exists and
    /// whose `next_direction` points back to the original connection, as well
    /// as at least one of the connections having `blocks` defined with a length
    /// greater than 0.
    ///
    /// If a direction is Up or Down, then the `attach_direction` must be
    /// defined and must be either North, South, West, or East.
    ///
    /// Returns the first error it finds.
    pub fn verify(&self) -> Result<(), String> {
        let mut connections = HashMap::new();

        if let Some(connection) = &self.connection_north {
            connections.insert(Direction::North, connection);
        }
        if let Some(connection) = &self.connection_south {
            connections.insert(Direction::South, connection);
        }
        if let Some(connection) = &self.connection_west {
            connections.insert(Direction::West, connection);
        }
        if let Some(connection) = &self.connection_east {
            connections.insert(Direction::East, connection);
        }
        if let Some(connection) = &self.connection_up {
            connections.insert(Direction::Up, connection);

            if connection.attach_direction.is_none() {
                return Err(
                    "The connection is Up, but the attach_direction is not defined".to_owned(),
                );
            }

            if let Some(attach_direction) = &connection.attach_direction {
                if !attach_direction.is_news() {
                    return Err(format!(
                        "The connection is Up, but the attach_direction is {:?}",
                        attach_direction,
                    ));
                }
            }
        }
        if let Some(connection) = &self.connection_down {
            connections.insert(Direction::Down, connection);

            if connection.attach_direction.is_none() {
                return Err(
                    "The connection is Down, but the attach_direction is not defined".to_owned(),
                );
            }

            if let Some(attach_direction) = &connection.attach_direction {
                if !attach_direction.is_news() {
                    return Err(format!(
                        "The connection is Down, but the attach_direction is {:?}",
                        attach_direction,
                    ));
                }
            }
        }

        if connections.len() < 2 {
            return Err("A tile must have at least two connections".to_owned());
        }

        for (direction, connection) in &connections {
            if let Some(next_connection) = connections.get(&connection.next_direction) {
                if next_connection.next_direction != *direction {
                    return Err(format!(
                        "The connection {:?} points to {:?}, but the connection {:?} points to {:?}",
                        direction,
                        connection.next_direction,
                        connection.next_direction,
                        next_connection.next_direction,
                    ));
                }

                if connection.blocks.is_none() && next_connection.blocks.is_none() {
                    return Err(format!(
                        "The connection {:?} and the connection {:?} both have no blocks",
                        direction, connection.next_direction,
                    ));
                }

                if let Some(blocks) = &connection.blocks {
                    if blocks.is_empty() {
                        return Err(format!("The connection {:?} has no blocks", direction,));
                    }
                }

                if let Some(blocks) = &next_connection.blocks {
                    if blocks.is_empty() {
                        return Err(format!(
                            "The connection {:?} has no blocks",
                            next_connection.next_direction,
                        ));
                    }
                }
            } else {
                return Err(format!(
                    "The connection {:?} points to {:?}, but that connection doesn't exist",
                    direction, connection.next_direction,
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ComplexGenerator {
    // TODO: Might want to use Rc<ComplexTile> instead of cloning.
    pub tile_size: BlockPos, // TODO: What to do if grid size is even?
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
        if tile_size.x % 2 == 0 || tile_size.z % 2 == 0 {
            panic!("Tile x & z size must be odd"); // TODO
        }

        let mut new_tiles = Vec::new();
        let origin = BlockPos::new(0, 0, tile_size.z / 2);
        for tile in tiles {
            if let Err(e) = tile.verify() {
                panic!("Invalid tile: {}", e);
            }
            new_tiles.extend(tile.get_all_rotations(origin, tile_size.x == tile_size.y));
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

    /// The `get_placement` function is used to select a tile based on the
    /// provided position, direction, and name. It checks if the next tile
    /// connects to the current tile and also ensures that it's not creating an
    /// infinite loop. If all checks pass, it returns the new position,
    /// direction, and the name of the connection for the next tile.
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

    /// The `get_placement` function is used to select a tile based on the
    /// provided position, direction, and name. It checks if the next tile
    /// connects to the current tile and also ensures that it's not creating an
    /// infinite loop. If all checks pass, it returns the new position,
    /// direction, and possible tiles that can be placed there.
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

            // If next direction is Up or Down, then we need to filter out tiles
            // that don't have the same attach_direction. We need to get the
            // attach_direction first.
            let attach_direction = tile
                .get_next(next_direction)
                .and_then(|c| c.attach_direction);

            // Filter out tiles that don't connect to the adjacent tiles
            // TODO: Move this to a separate function
            tiles = tiles
                .into_iter()
                .filter(|tile| {
                    // If the tile has an attach_direction, then it must match
                    // the attach_direction of the previous tile
                    if let Some(attach_direction) = attach_direction {
                        if let Some(Connection {
                            attach_direction: tile_attach_direction,
                            ..
                        }) = tile.get_next(next_direction.get_opposite())
                        {
                            if let Some(tile_attach_direction) = tile_attach_direction {
                                if tile_attach_direction != attach_direction {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
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

    /// The dfs function is the main driver of the path generation.
    ///
    /// It shuffles the possible tiles and then, for each tile, it checks if the
    /// position it leads to is within the grid boundaries and not visited yet.
    /// If the tile leads to the end of the grid, the function ends. If the tile
    /// doesn't lead to the end, the function calls itself recursively with the
    /// new position, direction, and possible tiles. If none of the tiles lead
    /// to a valid path, the function backtracks by removing the current
    /// position from the grid and adding it to the visited set.
    ///
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
            if pos.x < self.min_pos.x
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
                    if let Some(t) = self.dfs(new_pos, new_direction, new_tiles, visited) {
                        return Some(t);
                    }
                }
                None => {}
            }
            self.tile_grid.remove(&current_pos);
            visited.insert(pos);
        }
        None
    }

    pub fn generate_dfs(&mut self) -> Option<BlockPos> {
        let mut visited = HashSet::new();
        let current_pos = BlockPos::new(0, 0, 0);
        let current_direction = Direction::South;
        let current_tiles = self.starting_tiles.clone();

        return self.dfs(current_pos, current_direction, current_tiles, &mut visited);
    }

    pub fn get_block_segments(&self) -> Vec<Vec<BlockPos>> {
        let mut segments = Vec::new();
        let mut current_segment = Vec::new();
        let mut current_pos = BlockPos::new(0, 0, 0);
        let mut current_direction = Direction::South;

        loop {
            if let Some(tile) = self.get_tile(current_pos) {
                if let Some(Connection {
                    next_direction,
                    blocks,
                    ..
                }) = tile.get_next(current_direction.get_opposite())
                {
                    let has_blocks = if let Some(blocks) = blocks {
                        current_segment.extend(
                            blocks
                                .iter()
                                .map(|b| current_pos.mul_block_pos(self.tile_size) + *b),
                        );
                        true
                    } else {
                        false
                    };
                    if let Some(Connection { blocks, .. }) = tile.get_next(next_direction) {
                        if let Some(blocks) = blocks {
                            if has_blocks {
                                segments.push(current_segment);
                                current_segment = Vec::new();
                            }
                            current_segment.extend(
                                blocks
                                    .iter()
                                    .map(|b| current_pos.mul_block_pos(self.tile_size) + *b),
                            );
                        }
                    }
                    current_pos = current_pos + next_direction.to_block_pos();
                    current_direction = next_direction;
                    continue;
                } else {
                    panic!("Tile should have been filtered out if it doesn't have a connection");
                }
            } else {
                if !current_segment.is_empty() {
                    segments.push(current_segment);
                }
                break;
            }
        }
        segments
    }
}

impl BlockGenerator for ComplexGenerator {
    fn generate(&self, params: &BlockGenParams) -> GenerateResult {
        let mut blocks = HashMap::new();
        let mut children = Vec::new();

        for (pos, tile) in &self.tile_grid {
            let pos = pos.mul_block_pos(self.tile_size);

            tile.place(&mut blocks, &params.block_map.rebuild(), pos);
        }

        let segments = self.get_block_segments();

        for segment in segments {
            children.push(ChildGeneration::check_blocks(segment.into_iter().collect()));
        }

        children[0].reached = true;

        GenerateResult {
            start: BlockPos::new(0, 0, 0),
            end: BlockPos::new(0, 0, 0),
            blocks,
            children,
            alt_blocks: HashMap::new(),
            lines: Vec::new(),
        }
    }
}
