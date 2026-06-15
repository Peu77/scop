use std::ops::{Add, AddAssign, Div, Mul, Sub};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
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
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    values: [f32; 16],
}

impl Mat4 {
    pub const fn identity() -> Self {
        Self {
            values: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn translation(offset: Vec3) -> Self {
        let mut matrix = Self::identity();
        matrix.values[12] = offset.x;
        matrix.values[13] = offset.y;
        matrix.values[14] = offset.z;
        matrix
    }

    pub fn rotation_y(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            values: [
                cos, 0.0, -sin, 0.0,
                0.0, 1.0, 0.0, 0.0,
                sin, 0.0, cos, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn perspective(vertical_fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        let scale = 1.0 / (vertical_fov * 0.5).tan();
        Self {
            values: [
                scale / aspect,
                0.0,
                0.0,
                0.0,
                0.0,
                scale,
                0.0,
                0.0,
                0.0,
                0.0,
                (far + near) / (near - far),
                -1.0,
                0.0,
                0.0,
                (2.0 * far * near) / (near - far),
                0.0,
            ],
        }
    }

    pub fn as_ptr(&self) -> *const f32 {
        self.values.as_ptr()
    }
}

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut values = [0.0; 16];
        for column in 0..4 {
            for row in 0..4 {
                values[column * 4 + row] = (0..4)
                    .map(|index| self.values[index * 4 + row] * rhs.values[column * 4 + index])
                    .sum();
            }
        }
        Self { values }
    }
}

#[cfg(test)]
mod tests {
    use super::{Mat4, Vec3};

    #[test]
    fn identity_does_not_change_a_matrix() {
        let matrix = Mat4::translation(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(Mat4::identity() * matrix, matrix);
        assert_eq!(matrix * Mat4::identity(), matrix);
    }
}
