use rand::Rng;
use valence::{prelude::{DVec3, Vec3, Client}, BlockPos, protocol::Particle};

use crate::{utils::{get_edge_of_block, get_edge_of_block_dist}, line::Line3};

/*
 * Jump: net.minecraft.world.entity.LivingEntity: line ~1950
 *   - Jump Velocity: 0.42 * BlockJumpFactor + JumpBoostPower
 *     - BlockJumpFactor: Specific to the block being jumped on
 *     - JumpBoostPower: 0.1 * (Jump Boost Level + 1)
 *   - If sprinting, Horizontal Velocity += 0.2 (relative to direction)
 *
 * Horizontal Move: net.minecraft.world.entity.LivingEntity: lines 2080-2107 (travel)
 *   Note: xxa and zza are the player's input. xxa is forward/backward, zza is left/right.
 *         The function parameter is also the player's input.
 *   Speed is 0.13000001 when sprinting, 0.1 otherwise.
 *   Block friction is usually 0.6
 *   - If sprinting, Horizontal Velocity += 0.2 (relative to direction)
 *   - If sneaking, Horizontal Velocity *= 0.3
 */
const FRICTION: f32 = 0.91;
const BLOCK_FRICTION: f32 = 0.6;
const ON_GROUND: bool = false;
const SPEED: f32 = 0.13000001;
const FLYING_SPEED: f32 = 0.02;


const AVG_RUNNING_SPEED: f64 = 0.2873;
const AVG_RUN_JUMP_SPEED: f64 = 0.47;
const JUMP_VELOCITY: f64 = 0.42;
const JUMP_HEAD_HIT: f64 = 0.2;


#[derive(Debug, Clone, Copy)]
pub struct PlayerState {
    pub pos: DVec3,
    pub vel: DVec3,
    pub yaw: f32, // pitch doesn't matter for movement
    pub color: Vec3,
}

/// A player's state at a given point in time.
impl PlayerState {
    pub fn new(pos: DVec3, vel: DVec3, yaw: f32) -> Self {
        Self { 
            pos,
            vel,
            yaw,
            color: Vec3::new(
                rand::thread_rng().gen_range(0f32..1f32),
                rand::thread_rng().gen_range(0f32..1f32),
                rand::thread_rng().gen_range(0f32..1f32),
            ),
         }
    }

    pub fn running_jump(block_pos: BlockPos, yaw: f32) -> Self {
        let mut state = Self::new(get_edge_of_block(block_pos, yaw), DVec3::ZERO, yaw);
        state.pos.y += 1.;
        state.vel.x = -AVG_RUN_JUMP_SPEED * yaw.sin() as f64;
        state.vel.z = AVG_RUN_JUMP_SPEED * yaw.cos() as f64;
        state.vel.y = JUMP_VELOCITY;
        state
    }

    pub fn head_hit_jump(block_pos: BlockPos, yaw: f32) -> Self {
        let mut state = Self::new(get_edge_of_block_dist(block_pos, yaw, 1), DVec3::ZERO, yaw);
        state.vel.x = -AVG_RUNNING_SPEED * yaw.sin() as f64;
        state.vel.z = AVG_RUNNING_SPEED * yaw.cos() as f64;
        state.pos.y += 1. + JUMP_HEAD_HIT;
        state
    }

    pub fn get_block_pos(&self) -> BlockPos {
        BlockPos::new(self.pos.x.floor() as i32, self.pos.y.floor() as i32 - 1, self.pos.z.floor() as i32)
    }

    pub fn tick(&mut self) {
        let mut vel = self.handle_relative_friction_and_calculate_movement(self.get_accel());

        vel.y -= 0.08; // gravity
        vel.y *= 0.9800000190734863; // drag

        vel.x *= FRICTION as f64;
        vel.z *= FRICTION as f64;

        self.vel = vel;
    }

    fn draw_particle(&self, client: &mut Client) {
        client.play_particle(
            &Particle::Dust { rgb: self.color, scale: 1. },
            false,
            self.pos,
            Vec3::ZERO,
            0.0,
            1,
        );
    }

    pub fn draw_particles(&self, ticks: usize, client: &mut Client) {
        let mut state = self.clone();

        for _ in 0..ticks {
            state.draw_particle(client);
            state.tick();
        }
    }

    pub fn get_lines_for_number_of_ticks(&self, ticks: usize) -> Vec<Line3> {
        let mut state = self.clone();
        let mut pos = state.pos;
        let mut lines = Vec::new();

        for _ in 0..ticks {
            state.tick();
            let new_pos = state.pos;
            lines.push(Line3::new(pos.as_vec3(), new_pos.as_vec3()));
            pos = new_pos;
        }

        lines
    }

    pub fn get_state_in_ticks(&self, ticks: u32) -> Self {
        let mut state = self.clone();

        for _ in 0..ticks {
            state.tick();
        }

        state
    }

    fn get_accel(&self) -> DVec3 {
        let accel = 0.98f64;

        return DVec3::new(
            -accel * self.yaw.sin() as f64,
            0.0,
            accel * self.yaw.cos() as f64,
        );
    }

    fn handle_relative_friction_and_calculate_movement(&mut self, accel: DVec3) -> DVec3 {
        self.move_relative(self.get_friction_influenced_speed(BLOCK_FRICTION), accel);
        self.pos += self.vel;
        return self.vel;
    }

    fn move_relative(&mut self, speed: f32, accel: DVec3) {
        let vec3 = get_input_vector(accel, speed, self.yaw);
        self.vel += vec3;
    }

    fn get_friction_influenced_speed(&self, f: f32) -> f32 {
        if ON_GROUND {
            SPEED * (0.21600002f32 / (f * f * f))
        } else {
            FLYING_SPEED
        }
    }
}

fn get_input_vector(p_20016_: DVec3, p_20017_: f32, yaw: f32) -> DVec3 {
    let d0 = p_20016_.length_squared();
    if d0 < 1.0E-7 {
        DVec3::ZERO
    } else {
        let vec3 = if d0 > 1.0 {
            p_20016_.normalize()
        } else {
            p_20016_
        } * p_20017_ as f64;

        let f = yaw.sin();
        let f1 = yaw.cos();

        DVec3::new(
            vec3.x * f1 as f64 - vec3.z * f as f64,
            vec3.y,
            vec3.z * f1 as f64 + vec3.x * f as f64,
        )
    }
}
