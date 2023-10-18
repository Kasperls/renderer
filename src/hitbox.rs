use cgmath::{Vector3, Zero};

// create some form of raw hitbox
pub struct HitBoxRaw {
    pos: Vector3<f32>,
    size: f32,
}

impl HitBoxRaw {
    pub fn new(pos: Vector3<f32>, size: f32) -> Self {
        Self { pos, size }
    }
}

impl HitBox for &HitBoxRaw {
    fn negative_corner(&self) -> Vector3<f32> {
        Vector3::new(
            self.pos.x - self.size / 2.0,
            self.pos.y - self.size / 2.0,
            self.pos.z - self.size / 2.0,
        )
    }

    fn positive_corner(&self) -> Vector3<f32> {
        Vector3::new(
            self.pos.x + self.size / 2.0,
            self.pos.y + self.size / 2.0,
            self.pos.z + self.size / 2.0,
        )
    }
}


pub trait HitBox {
    fn negative_corner(&self) -> Vector3<f32>;
    fn positive_corner(&self) -> Vector3<f32>;
    fn hitbox_collision<T>(&self, other: T) -> bool where T: HitBox {
        if self.negative_corner().x < other.positive_corner().x && self.positive_corner().x > other.negative_corner().x &&
            self.negative_corner().y < other.positive_corner().y && self.positive_corner().y > other.negative_corner().y &&
            self.negative_corner().z < other.positive_corner().z && self.positive_corner().z > other.negative_corner().z {
                return true;
            }
        false
    }
}

pub struct HitPlane {
    top_left: Vector3<f32>,
    bottom_right: Vector3<f32>,
}