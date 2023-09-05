use valence::{BlockPos, prelude::{DVec3, Client, Vec3}, protocol::Particle};

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
        &Particle::Dust { rgb: color, scale: 1. },
        false,
        pos,
        Vec3::ZERO,
        0.0,
        1,
    );
}
