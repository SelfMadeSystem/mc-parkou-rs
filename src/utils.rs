#![allow(dead_code)]

use rand::Rng;
use valence::{
    math::IVec3,
    prelude::{Client, DVec3, Vec3},
    protocol::Particle,
    BlockPos,
};

use crate::{line::Line3, prediction::prediction_state::PredictionState};

pub const PLAYER_WIDTH: f64 = 0.6;
pub const PLAYER_HEIGHT: f64 = 1.8;

pub fn get_edge_of_block(pos: BlockPos, yaw: f32) -> DVec3 {
    get_edge_of_block_dist(pos, yaw, 0)
}

pub fn get_edge_of_block_dist(pos: BlockPos, yaw: f32, dist: impl Into<f64>) -> DVec3 {
    let mut pos = DVec3::new(pos.x as f64, pos.y as f64, pos.z as f64);
    pos.x += 0.5;
    pos.z += 0.5;
    let add = DVec3::new(-yaw.sin() as f64, 0.0, yaw.cos() as f64);
    pos + add * dist.into() // not optimal. does circle instead of square
}

#[allow(dead_code)]
pub fn particle_outline_block(pos: BlockPos, color: Vec3, client: &mut Client) {
    let pos = DVec3::new(pos.x as f64, pos.y as f64, pos.z as f64);

    const AMOUNT: usize = 2;

    for i in 0..=AMOUNT {
        let f = i as f64 / AMOUNT as f64;

        {
            let mut pos = pos;
            pos.x += f;

            draw_particle(client, color, pos);
            pos.y += 1.;
            draw_particle(client, color, pos);
            pos.z += 1.;
            draw_particle(client, color, pos);
            pos.y -= 1.;
            draw_particle(client, color, pos);
        }

        {
            let mut pos = pos;
            pos.y += f;

            draw_particle(client, color, pos);
            pos.x += 1.;
            draw_particle(client, color, pos);
            pos.z += 1.;
            draw_particle(client, color, pos);
            pos.x -= 1.;
            draw_particle(client, color, pos);
        }

        {
            let mut pos = pos;
            pos.z += f;

            draw_particle(client, color, pos);
            pos.y += 1.;
            draw_particle(client, color, pos);
            pos.x += 1.;
            draw_particle(client, color, pos);
            pos.y -= 1.;
            draw_particle(client, color, pos);
        }
    }
}

fn draw_particle(client: &mut Client, color: Vec3, pos: DVec3) {
    client.play_particle(
        &Particle::Dust {
            rgb: color,
            scale: 1.,
        },
        false,
        pos,
        Vec3::ZERO,
        0.0,
        1,
    );
}

#[allow(dead_code)]
pub fn get_lines_for_block(pos: BlockPos) -> Vec<Line3> {
    let mut lines = Vec::new();

    let pos = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);

    lines.push(Line3::new(pos, pos + Vec3::new(1., 0., 0.)));
    lines.push(Line3::new(pos, pos + Vec3::new(0., 1., 0.)));
    lines.push(Line3::new(pos, pos + Vec3::new(0., 0., 1.)));

    lines.push(Line3::new(
        pos + Vec3::new(1., 0., 0.),
        pos + Vec3::new(1., 1., 0.),
    ));
    lines.push(Line3::new(
        pos + Vec3::new(1., 0., 0.),
        pos + Vec3::new(1., 0., 1.),
    ));

    lines.push(Line3::new(
        pos + Vec3::new(0., 1., 0.),
        pos + Vec3::new(1., 1., 0.),
    ));
    lines.push(Line3::new(
        pos + Vec3::new(0., 1., 0.),
        pos + Vec3::new(0., 1., 1.),
    ));

    lines.push(Line3::new(
        pos + Vec3::new(0., 0., 1.),
        pos + Vec3::new(1., 0., 1.),
    ));
    lines.push(Line3::new(
        pos + Vec3::new(0., 0., 1.),
        pos + Vec3::new(0., 1., 1.),
    ));

    lines.push(Line3::new(
        pos + Vec3::new(1., 1., 0.),
        pos + Vec3::new(1., 1., 1.),
    ));
    lines.push(Line3::new(
        pos + Vec3::new(1., 0., 1.),
        pos + Vec3::new(1., 1., 1.),
    ));
    lines.push(Line3::new(
        pos + Vec3::new(0., 1., 1.),
        pos + Vec3::new(1., 1., 1.),
    ));

    lines
}

pub fn random_yaw() -> f32 {
    random_yaw_dist(60.0)
}

pub fn random_yaw_dist(f: impl Into<f32>) -> f32 {
    let f = f.into();
    rand::thread_rng().gen_range(-f..f).to_radians()
}

pub fn get_blocks_between(start: Vec3, end: Vec3) -> Vec<BlockPos> {
    let mut blocks = Vec::new();

    let gx0 = start.x;
    let gy0 = start.y;
    let gz0 = start.z;

    let gx1 = end.x;
    let gy1 = end.y;
    let gz1 = end.z;

    let gx0idx = gx0.floor() as i32;
    let gy0idx = gy0.floor() as i32;
    let gz0idx = gz0.floor() as i32;

    let gx1idx = gx1.floor() as i32;
    let gy1idx = gy1.floor() as i32;
    let gz1idx = gz1.floor() as i32;

    let sx = if gx1idx > gx0idx {
        1
    } else {
        if gx1idx < gx0idx {
            -1
        } else {
            0
        }
    };
    let sy = if gy1idx > gy0idx {
        1
    } else {
        if gy1idx < gy0idx {
            -1
        } else {
            0
        }
    };
    let sz = if gz1idx > gz0idx {
        1
    } else {
        if gz1idx < gz0idx {
            -1
        } else {
            0
        }
    };

    let mut gx = gx0idx;
    let mut gy = gy0idx;
    let mut gz = gz0idx;

    //Planes for each axis that we will next cross
    let gxp = gx0idx + (if gx1idx > gx0idx { 1 } else { 0 });
    let gyp = gy0idx + (if gy1idx > gy0idx { 1 } else { 0 });
    let gzp = gz0idx + (if gz1idx > gz0idx { 1 } else { 0 });

    //Only used for multiplying up the error margins
    let vx = if gx1 == gx0 { 1. } else { gx1 - gx0 };
    let vy = if gy1 == gy0 { 1. } else { gy1 - gy0 };
    let vz = if gz1 == gz0 { 1. } else { gz1 - gz0 };

    //Error is normalized to vx * vy * vz so we only have to multiply up
    let vxvy = vx * vy;
    let vxvz = vx * vz;
    let vyvz = vy * vz;

    //Error from the next plane accumulators, scaled up by vx*vy*vz
    // gx0 + vx * rx == gxp
    // vx * rx == gxp - gx0
    // rx == (gxp - gx0) / vx
    let mut errx = (gxp as f32 - gx0) * vyvz;
    let mut erry = (gyp as f32 - gy0) * vxvz;
    let mut errz = (gzp as f32 - gz0) * vxvy;

    let derrx = sx as f32 * vyvz;
    let derry = sy as f32 * vxvz;
    let derrz = sz as f32 * vxvy;

    loop {
        blocks.push(BlockPos::new(gx, gy, gz));

        if gx == gx1idx && gy == gy1idx && gz == gz1idx {
            break;
        }

        //Which plane do we cross first?
        let xr = errx.abs();
        let yr = erry.abs();
        let zr = errz.abs();

        if sx != 0 && (sy == 0 || xr < yr) && (sz == 0 || xr < zr) {
            gx += sx;
            errx += derrx;
        } else if sy != 0 && (sz == 0 || yr < zr) {
            gy += sy;
            erry += derry;
        } else if sz != 0 {
            gz += sz;
            errz += derrz;
        }
    }

    blocks
}

pub fn prediction_can_reach(from: DVec3, to: BlockPos) -> bool {
    let yaw = (to.x as f64 - from.x).atan2(to.z as f64 - from.z) as f32;

    let mut state = PredictionState::running_jump_vec(from, yaw);

    loop {
        let pos = state.pos.as_block_pos();

        if pos.y >= to.y && pos.x >= to.x && pos.z >= to.z && pos.x <= to.x + 1 && pos.z <= to.z + 1
        {
            return true;
        }

        if pos.y < to.y && state.vel.y < 0.0 {
            return false;
        }

        state.tick();
    }
}

pub fn get_player_floor_blocks(mut pos: DVec3) -> Vec<BlockPos> {
    let mut blocks = Vec::new();

    if pos.y % 1. == 0. {
        pos.y -= 1.;
    }

    let x0 = pos.x - PLAYER_WIDTH / 2.;
    let x1 = pos.x + PLAYER_WIDTH / 2.;

    let z0 = pos.z - PLAYER_WIDTH / 2.;
    let z1 = pos.z + PLAYER_WIDTH / 2.;

    let y = pos.y.floor() as i32;

    let x0idx = x0.floor() as i32;
    let x1idx = x1.floor() as i32;

    let z0idx = z0.floor() as i32;
    let z1idx = z1.floor() as i32;

    for x in x0idx..=x1idx {
        for z in z0idx..=z1idx {
            blocks.push(BlockPos::new(x, y, z));
        }
    }

    blocks
}

pub fn get_min_max_yaw(prev: BlockPos, size: &IVec3) -> (f32, f32) {
    const DIST: f32 = 5.;

    let min_yaw = if prev.x as f32 - 1. >= DIST {
        999.
    } else {
        std::f32::consts::PI / 2. - ((prev.x as f32 - 1.) / DIST).acos()
    }
    .min(45f32.to_radians())
        * -1.;

    let max_yaw = if (size.x - 2 - prev.x) as f32 >= DIST {
        999.
    } else {
        std::f32::consts::PI / 2. - ((size.x - 2 - prev.x) as f32 / DIST).acos()
    }
    .min(45f32.to_radians());
    (min_yaw, max_yaw)
}

/// Gets the four directions next to the given direction.
///
/// The given direction must be a unit vector.
///
/// For example, if the direction is (1, 0, 0), then the returned array will contain:
/// - (0, 1, 0)
/// - (0, -1, 0)
/// - (0, 0, 1)
/// - (0, 0, -1)
pub fn get_dirs_next_to(dir: BlockPos) -> [BlockPos; 4] {
    if dir.y != 0 {
        [
            BlockPos::new(1, 0, 0),
            BlockPos::new(-1, 0, 0),
            BlockPos::new(0, 0, 1),
            BlockPos::new(0, 0, -1),
        ]
    } else if dir.x != 0 {
        [
            BlockPos::new(0, 1, 0),
            BlockPos::new(0, -1, 0),
            BlockPos::new(0, 0, 1),
            BlockPos::new(0, 0, -1),
        ]
    } else {
        [
            BlockPos::new(0, 1, 0),
            BlockPos::new(0, -1, 0),
            BlockPos::new(1, 0, 0),
            BlockPos::new(-1, 0, 0),
        ]
    }
}

pub fn random_sign() -> i32 {
    if rand::thread_rng().gen() {
        1
    } else {
        -1
    }
}

#[derive(Clone, Copy, Debug)]
pub enum JumpDirection {
    Up,
    Down,
    DoesntMatter,
}

impl JumpDirection {
    pub fn get_y_offset(self) -> i32 {
        match self {
            JumpDirection::Up => 1,
            JumpDirection::Down => -rand::thread_rng().gen_range(1..=2),
            JumpDirection::DoesntMatter => rand::thread_rng().gen_range(-1..=1),
        }
    }

    pub fn go_down(self) -> bool {
        match self {
            JumpDirection::Up => false,
            JumpDirection::Down => true,
            JumpDirection::DoesntMatter => rand::thread_rng().gen(),
        }
    }

    pub fn go_up(self) -> bool {
        !self.go_down()
    }
}

pub trait AsBlockPos {
    fn as_block_pos(&self) -> BlockPos;
}

impl AsBlockPos for DVec3 {
    fn as_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x as i32, self.y as i32, self.z as i32)
    }
}

impl AsBlockPos for Vec3 {
    fn as_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x as i32, self.y as i32, self.z as i32)
    }
}

pub trait ToVec3 {
    fn to_vec3(&self) -> Vec3;
}

impl ToVec3 for BlockPos {
    fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

pub trait DVec3With {
    fn with_x(&self, x: f64) -> DVec3;
    fn with_y(&self, y: f64) -> DVec3;
    fn with_z(&self, z: f64) -> DVec3;
}

impl DVec3With for DVec3 {
    fn with_x(&self, x: f64) -> DVec3 {
        DVec3::new(x, self.y, self.z)
    }

    fn with_y(&self, y: f64) -> DVec3 {
        DVec3::new(self.x, y, self.z)
    }

    fn with_z(&self, z: f64) -> DVec3 {
        DVec3::new(self.x, self.y, z)
    }
}
