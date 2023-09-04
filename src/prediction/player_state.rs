use valence::prelude::DVec3;

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

pub struct PlayerState {
    pub pos: DVec3,
    pub vel: DVec3,
    pub yaw: f32, // pitch doesn't matter for movement
}

/// A player's state at a given point in time.
impl PlayerState {
    pub fn new(pos: DVec3, vel: DVec3, yaw: f32) -> Self {
        Self { pos, vel, yaw }
    }

    pub fn tick(&mut self) {
        let mut vel = self.handle_relative_friction_and_calculate_movement(self.get_accel());

        vel.y -= 0.08; // gravity
        vel.y *= 0.9800000190734863; // drag

        vel.x *= FRICTION as f64;
        vel.y *= FRICTION as f64;

        self.vel = vel;
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

fn get_input_vector(p_20016_: DVec3, p_20017_: f32, p_20018_: f32) -> DVec3 {
    let d0 = p_20016_.length_squared();
    if d0 < 1.0E-7 {
        DVec3::ZERO
    } else {
        let vec3 = if d0 > 1.0 {
            p_20016_.normalize()
        } else {
            p_20016_
        } * p_20017_ as f64;

        let f = (p_20018_ *  std::f32::consts::PI / 180.0).sin();
        let f1 = (p_20018_ * std::f32::consts::PI / 180.0).cos();

        DVec3::new(
            vec3.x * f1 as f64 - vec3.z * f as f64,
            vec3.y,
            vec3.z * f1 as f64 + vec3.x * f as f64,
        )
    }
}
