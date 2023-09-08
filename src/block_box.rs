use valence::prelude::*;

/// Represents a box of blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockBox {
    min: BlockPos,
    max: BlockPos,
}

#[allow(dead_code)]
impl BlockBox {
    pub fn new(min: BlockPos, max: BlockPos) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> BlockPos {
        BlockPos::new(
            (self.min.x + self.max.x) / 2,
            (self.min.y + self.max.y) / 2,
            (self.min.z + self.max.z) / 2,
        )
    }

    /// Gets the furthest central position in the given direction.
    pub fn furthest_center(&self, direction: Direction) -> BlockPos {
        let center = self.center();

        match direction {
            Direction::North => BlockPos::new(center.x, center.y, self.min.z),
            Direction::South => BlockPos::new(center.x, center.y, self.max.z),
            Direction::East => BlockPos::new(self.max.x, center.y, center.z),
            Direction::West => BlockPos::new(self.min.x, center.y, center.z),
            Direction::Up => BlockPos::new(center.x, self.max.y, center.z),
            Direction::Down => BlockPos::new(center.x, self.min.y, center.z),
        }
    }

    pub fn contains(&self, pos: BlockPos) -> bool {
        pos.x >= self.min.x
            && pos.x <= self.max.x
            && pos.y >= self.min.y
            && pos.y <= self.max.y
            && pos.z >= self.min.z
            && pos.z <= self.max.z
    }

    pub fn contains_player(&self, pos: Position) -> bool {
        let pos = BlockPos::new(
            (pos.0.x - 0.5).round() as i32,
            pos.0.y as i32 - 1,
            (pos.0.z - 0.5).round() as i32,
        );

        self.contains(pos)
    }

    pub fn expand_to(&self, pos: BlockPos) -> BlockBox {
        BlockBox::new(
            BlockPos::new(
                self.min.x.min(pos.x),
                self.min.y.min(pos.y),
                self.min.z.min(pos.z),
            ),
            BlockPos::new(
                self.max.x.max(pos.x),
                self.max.y.max(pos.y),
                self.max.z.max(pos.z),
            ),
        )
    }
}