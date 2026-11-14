use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Returns the zero vector when the length is zero.
    pub fn normalize(self) -> Self {
        let length = self.length();
        if length > 0.0 {
            self * (1.0 / length)
        } else {
            Self::ZERO
        }
    }

    pub fn min(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    pub fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    pub const fn to_array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self::new(x, y, z)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

/// Column-major, like glTF.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4(pub [f32; 16]);

impl Default for Mat4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mat4 {
    pub const IDENTITY: Self = Self([
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 1.0,
    ]);

    pub const fn translation(x: f32, y: f32, z: f32) -> Self {
        let mut matrix = Self::IDENTITY;
        matrix.0[12] = x;
        matrix.0[13] = y;
        matrix.0[14] = z;
        matrix
    }

    pub const fn scale(x: f32, y: f32, z: f32) -> Self {
        let mut matrix = Self::IDENTITY;
        matrix.0[0] = x;
        matrix.0[5] = y;
        matrix.0[10] = z;
        matrix
    }

    pub fn rotation_axis(axis: Vec3, radians: f32) -> Self {
        let Vec3 { x, y, z } = axis.normalize();
        let (sin, cos) = radians.sin_cos();
        let t = 1.0 - cos;
        Self([
            t * x * x + cos,
            t * x * y + sin * z,
            t * x * z - sin * y,
            0.0,
            t * x * y - sin * z,
            t * y * y + cos,
            t * y * z + sin * x,
            0.0,
            t * x * z + sin * y,
            t * y * z - sin * x,
            t * z * z + cos,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    /// `rotation` is an `[x, y, z, w]` quaternion.
    pub fn from_trs(
        translation: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    ) -> Self {
        let [x, y, z, w] = rotation;
        let (x2, y2, z2) = (x + x, y + y, z + z);
        let (xx, xy, xz) = (x * x2, x * y2, x * z2);
        let (yy, yz, zz) = (y * y2, y * z2, z * z2);
        let (wx, wy, wz) = (w * x2, w * y2, w * z2);
        let [sx, sy, sz] = scale;
        Self([
            (1.0 - (yy + zz)) * sx,
            (xy + wz) * sx,
            (xz - wy) * sx,
            0.0,
            (xy - wz) * sy,
            (1.0 - (xx + zz)) * sy,
            (yz + wx) * sy,
            0.0,
            (xz + wy) * sz,
            (yz - wx) * sz,
            (1.0 - (xx + yy)) * sz,
            0.0,
            translation[0],
            translation[1],
            translation[2],
            1.0,
        ])
    }

    /// Right-handed.
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let forward = (eye - target).normalize();
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);
        Self([
            right.x,
            up.x,
            forward.x,
            0.0,
            right.y,
            up.y,
            forward.y,
            0.0,
            right.z,
            up.z,
            forward.z,
            0.0,
            -right.dot(eye),
            -up.dot(eye),
            -forward.dot(eye),
            1.0,
        ])
    }

    /// Right-handed, with depth mapped to `-1..=1`.
    pub fn perspective(
        fov_y_radians: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let f = 1.0 / (fov_y_radians / 2.0).tan();
        let mut matrix = Self([0.0; 16]);
        matrix.0[0] = f / aspect;
        matrix.0[5] = f;
        matrix.0[10] = (far + near) / (near - far);
        matrix.0[11] = -1.0;
        matrix.0[14] = 2.0 * far * near / (near - far);
        matrix
    }

    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        let [x, y, z, _] = self.transform_point4(point);
        Vec3::new(x, y, z)
    }

    pub fn transform_point4(&self, point: Vec3) -> [f32; 4] {
        let m = &self.0;
        [
            m[0] * point.x + m[4] * point.y + m[8] * point.z + m[12],
            m[1] * point.x + m[5] * point.y + m[9] * point.z + m[13],
            m[2] * point.x + m[6] * point.y + m[10] * point.z + m[14],
            m[3] * point.x + m[7] * point.y + m[11] * point.z + m[15],
        ]
    }

    pub const fn translation_part(&self) -> Vec3 {
        Vec3::new(self.0[12], self.0[13], self.0[14])
    }
}

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut out = [0.0; 16];
        for column in 0..4 {
            for row in 0..4 {
                out[column * 4 + row] = (0..4)
                    .map(|k| self.0[k * 4 + row] * other.0[column * 4 + k])
                    .sum();
            }
        }
        Self(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: Vec3, b: Vec3) {
        assert!((a - b).length() < 1e-5, "{a:?} != {b:?}");
    }

    #[test]
    fn composes_transforms_right_to_left() {
        let matrix =
            Mat4::translation(1.0, 2.0, 3.0) * Mat4::scale(2.0, 2.0, 2.0);
        assert_close(
            matrix.transform_point(Vec3::new(1.0, 1.0, 1.0)),
            Vec3::new(3.0, 4.0, 5.0),
        );
    }

    #[test]
    fn axis_rotation_matches_quaternion_rotation() {
        let angle = 0.7_f32;
        let axis = Vec3::new(1.0, 0.0, 0.0);
        let quaternion = [(angle / 2.0).sin(), 0.0, 0.0, (angle / 2.0).cos()];
        let point = Vec3::new(0.3, 0.5, -0.2);
        assert_close(
            Mat4::rotation_axis(axis, angle).transform_point(point),
            Mat4::from_trs([0.0; 3], quaternion, [1.0; 3])
                .transform_point(point),
        );
    }

    #[test]
    fn look_at_places_the_target_on_the_negative_z_axis() {
        let view = Mat4::look_at(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::Y,
        );
        assert_close(
            view.transform_point(Vec3::new(0.0, 0.0, 1.0)),
            Vec3::new(0.0, 0.0, -4.0),
        );
    }
}
