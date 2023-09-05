use valence::{BlockPos, prelude::DVec3};

pub fn get_edge_of_block(pos: BlockPos, yaw: f32) -> DVec3 {
    get_edge_of_block_dist(pos, yaw, 0.25)
}

pub fn get_edge_of_block_dist(pos: BlockPos, yaw: f32, dist: impl Into<f64>) -> DVec3 {
    let mut pos = DVec3::new(pos.x as f64, pos.y as f64, pos.z as f64);
    pos.x += 0.5;
    pos.z += 0.5;
    let add = DVec3::new(-yaw.sin() as f64, 0.0, yaw.cos() as f64);
    pos + add * dist.into() // not optimal. does circle instead of square
}