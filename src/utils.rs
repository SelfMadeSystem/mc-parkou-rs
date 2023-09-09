use rand::Rng;
use valence::{
    prelude::{Client, DVec3, Vec3},
    protocol::Particle,
    BlockPos,
};

use crate::line::Line3;

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

    lines.push(Line3::new(pos + Vec3::new(1., 0., 0.), pos + Vec3::new(1., 1., 0.)));
    lines.push(Line3::new(pos + Vec3::new(1., 0., 0.), pos + Vec3::new(1., 0., 1.)));

    lines.push(Line3::new(pos + Vec3::new(0., 1., 0.), pos + Vec3::new(1., 1., 0.)));
    lines.push(Line3::new(pos + Vec3::new(0., 1., 0.), pos + Vec3::new(0., 1., 1.)));

    lines.push(Line3::new(pos + Vec3::new(0., 0., 1.), pos + Vec3::new(1., 0., 1.)));
    lines.push(Line3::new(pos + Vec3::new(0., 0., 1.), pos + Vec3::new(0., 1., 1.)));

    lines.push(Line3::new(pos + Vec3::new(1., 1., 0.), pos + Vec3::new(1., 1., 1.)));
    lines.push(Line3::new(pos + Vec3::new(1., 0., 1.), pos + Vec3::new(1., 1., 1.)));
    lines.push(Line3::new(pos + Vec3::new(0., 1., 1.), pos + Vec3::new(1., 1., 1.)));

    lines
}

pub fn to_rad(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}

pub fn to_deg(rad: f32) -> f32 {
    rad * 180.0 / std::f32::consts::PI
}

pub fn random_yaw() -> f32 {
    random_yaw_dist(60.0)
}

pub fn random_yaw_dist(f: impl Into<f32>) -> f32 {
    let f = f.into();
    to_rad(rand::thread_rng().gen_range(-f..f))
}

#[derive(Clone, Copy, Debug)]
pub enum JumpDirection {
    Up,
    Down,
    DoesntMatter,
}
