use valence::{
    entity::{
        block_display::BlockDisplayEntityBundle,
        display::{LeftRotation, Scale}, entity::Flags,
    },
    math::Quat,
    prelude::*,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Line3 {
    pub start: Vec3,
    pub end: Vec3,
}

impl Eq for Line3 {
}

// Needed because Vec3 doesn't implement Hash
impl std::hash::Hash for Line3 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let start = self.start;
        let end = self.end;

        start.x.to_bits().hash(state);
        start.y.to_bits().hash(state);
        start.z.to_bits().hash(state);

        end.x.to_bits().hash(state);
        end.y.to_bits().hash(state);
        end.z.to_bits().hash(state);
    }
}

impl Line3 {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self { start, end }
    }

    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_arc(
            Vec3::new(0., 0., 1.),
            (self.end - self.start).normalize(),
        )
    }

    pub fn to_block_display(&self) -> BlockDisplayEntityBundle {
        let mut bundle = BlockDisplayEntityBundle::default();
        bundle.position = Position(self.start.as_dvec3());
        const WIDTH: f32 = 0.05;
        bundle.display_scale = Scale(Vec3::new(WIDTH, WIDTH, self.start.distance(self.end)));
        bundle.display_left_rotation = LeftRotation(self.rotation());
        // bundle.display_translation = Translation(Vec3::new(0.5, 0.5, 0.5));
        bundle.entity_flags = {
            let mut flags = Flags::default();
            flags.set_glowing(true);
            flags
        };

        bundle
    }
}

impl std::ops::Add<Vec3> for Line3 {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self {
            start: self.start + rhs,
            end: self.end + rhs,
        }
    }
}
