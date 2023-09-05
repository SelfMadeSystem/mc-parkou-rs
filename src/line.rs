use valence::{prelude::*, entity::{block_display::BlockDisplayEntityBundle, display::{Translation, Scale, LeftRotation}}, math::Quat};

#[derive(Copy, Clone, Debug)]
pub struct Line3 {
    pub start: Vec3,
    pub end: Vec3,
}

impl Line3 {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self { start, end }
    }

    /*
     * function quaternionFromVectors(u, v) {
     *     d = dotProduct(u, v);
     *     w = crossProduct(u, v);
     *     
     *     return normalizeQuaternion([d + sqrt(d * d + dotProduct(w, w)), w]);
     * }
     */
    pub fn rotation(&self) -> Quat {
        let d = self.start.dot(self.end);
        let w = self.start.cross(self.end);

        Quat::from_xyzw(
            d + (d * d + w.dot(w)).sqrt(),
            w.x,
            w.y,
            w.z,
        ).normalize()   
    }

    pub fn to_block_display(&self) -> BlockDisplayEntityBundle {
        let mut bundle = BlockDisplayEntityBundle::default();
        bundle.position = Position(self.start.as_dvec3());
        bundle.display_scale = Scale(Vec3::new(0.1, 0.1, self.start.distance(self.end)));
        bundle.display_left_rotation = LeftRotation(self.rotation());

        bundle
    }
}