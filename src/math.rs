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

    pub fn normalized(self) -> Self {
        let length = self.length();
        if length <= f32::EPSILON {
            Self::new(0.0, 0.0, 1.0)
        } else {
            self / length
        }
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
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
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
                cos, 0.0, -sin, 0.0, 0.0, 1.0, 0.0, 0.0, sin, 0.0, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn rotation_x(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            values: [
                1.0, 0.0, 0.0, 0.0, 0.0, cos, sin, 0.0, 0.0, -sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
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
    use super::{Mat4, Vec2, Vec3};

    const EPSILON: f32 = 1.0e-6;

    fn assert_f32_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_matrix_close(actual: Mat4, expected: [f32; 16]) {
        for (actual, expected) in actual.values.into_iter().zip(expected) {
            assert_f32_close(actual, expected);
        }
    }

    #[test]
    fn vec2_new_sets_both_components() {
        assert_eq!(Vec2::new(1.5, -2.0), Vec2 { x: 1.5, y: -2.0 });
    }

    #[test]
    fn vec3_adds_component_wise() {
        let result = Vec3::new(1.0, -2.0, 3.5) + Vec3::new(4.0, 2.5, -1.5);

        assert_eq!(result, Vec3::new(5.0, 0.5, 2.0));
    }

    #[test]
    fn vec3_add_assign_updates_each_component() {
        let mut value = Vec3::new(1.0, 2.0, 3.0);

        value += Vec3::new(-1.0, 4.0, 0.5);

        assert_eq!(value, Vec3::new(0.0, 6.0, 3.5));
    }

    #[test]
    fn vec3_subtracts_component_wise() {
        let result = Vec3::new(5.0, 3.0, -2.0) - Vec3::new(2.0, 4.0, -5.0);

        assert_eq!(result, Vec3::new(3.0, -1.0, 3.0));
    }

    #[test]
    fn vec3_multiplies_each_component_by_scalar() {
        assert_eq!(Vec3::new(1.5, -2.0, 4.0) * 2.0, Vec3::new(3.0, -4.0, 8.0));
    }

    #[test]
    fn vec3_divides_each_component_by_scalar() {
        assert_eq!(Vec3::new(3.0, -4.0, 8.0) / 2.0, Vec3::new(1.5, -2.0, 4.0));
    }

    #[test]
    fn vec3_min_selects_smallest_component_values() {
        let result = Vec3::new(1.0, 5.0, -3.0).min(Vec3::new(2.0, 4.0, -5.0));

        assert_eq!(result, Vec3::new(1.0, 4.0, -5.0));
    }

    #[test]
    fn vec3_max_selects_largest_component_values() {
        let result = Vec3::new(1.0, 5.0, -3.0).max(Vec3::new(2.0, 4.0, -5.0));

        assert_eq!(result, Vec3::new(2.0, 5.0, -3.0));
    }

    #[test]
    fn identity_is_neutral_on_left_side() {
        let matrix = Mat4::translation(Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(Mat4::identity() * matrix, matrix);
    }

    #[test]
    fn identity_is_neutral_on_right_side() {
        let matrix = Mat4::translation(Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(matrix * Mat4::identity(), matrix);
    }

    #[test]
    fn translation_places_offset_in_last_column() {
        let matrix = Mat4::translation(Vec3::new(2.0, -3.0, 4.5));

        assert_eq!(
            matrix.values,
            [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, -3.0, 4.5, 1.0,]
        );
    }

    #[test]
    fn multiplying_translations_combines_their_offsets() {
        let first = Mat4::translation(Vec3::new(1.0, 2.0, 3.0));
        let second = Mat4::translation(Vec3::new(-4.0, 5.0, 1.0));

        assert_eq!(first * second, Mat4::translation(Vec3::new(-3.0, 7.0, 4.0)));
    }

    #[test]
    fn rotation_y_by_quarter_turn_uses_column_major_layout() {
        let matrix = Mat4::rotation_y(std::f32::consts::FRAC_PI_2);

        assert_matrix_close(
            matrix,
            [
                0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        );
    }

    #[test]
    fn rotation_x_by_quarter_turn_uses_column_major_layout() {
        let matrix = Mat4::rotation_x(std::f32::consts::FRAC_PI_2);

        assert_matrix_close(
            matrix,
            [
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        );
    }

    #[test]
    fn perspective_builds_expected_projection_matrix() {
        let matrix = Mat4::perspective(std::f32::consts::FRAC_PI_2, 2.0, 1.0, 11.0);

        assert_matrix_close(
            matrix,
            [
                0.5, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.2, -1.0, 0.0, 0.0, -2.2, 0.0,
            ],
        );
    }

    #[test]
    fn as_ptr_points_to_first_matrix_value() {
        let matrix = Mat4::translation(Vec3::new(2.0, 3.0, 4.0));

        assert_eq!(matrix.as_ptr(), matrix.values.as_ptr());
    }
}
