//! This module adds support to [`f32`], [`f64`], [`Vec2`] and [`Vec3`] for
//! approximate comparison operations.

use bevy::math::{Vec2, Vec3};

/// Trait for types that can be approximately compared. This is useful for
/// comparing floating point numbers, which are often not exactly equal due to
/// floating point precision errors.
pub trait ApproxEq {
    /// The epsilon value to use for approximate comparisons.
    const EPSILON: f32 = 1e-6;

    /// Returns `true` if `self` is approximately equal to `other`. Two floating
    /// point numbers are considered approximately equal if their difference is
    /// less than [`Self::EPSILON`].
    fn approx_eq(&self, other: &Self) -> bool;
}

impl ApproxEq for f32 {
    fn approx_eq(&self, other: &Self) -> bool {
        (self - other).abs() < <Self as ApproxEq>::EPSILON
    }
}

impl ApproxEq for f64 {
    fn approx_eq(&self, other: &Self) -> bool {
        (self - other).abs() < <Self as ApproxEq>::EPSILON as f64
    }
}

impl ApproxEq for Vec2 {
    fn approx_eq(&self, other: &Self) -> bool {
        self.x.approx_eq(&other.x) && self.y.approx_eq(&other.y)
    }
}

impl ApproxEq for Vec3 {
    fn approx_eq(&self, other: &Self) -> bool {
        self.x.approx_eq(&other.x) && self.y.approx_eq(&other.y) && self.z.approx_eq(&other.z)
    }
}

/// A macro for asserting that two values are approximately equal. This is
/// useful for comparing floating point numbers, which are often not exactly
/// equal due to floating point precision errors.
///
/// Epsilon value is defined by [`ApproxEq::EPSILON`].
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr) => {
        assert!(
            $crate::utilities::vec_cmp::ApproxEq::approx_eq(&$left, &$right),
            "{:?} is not approximately equal to {:?}",
            $left,
            $right
        );
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn f32_approx_eq() {
        assert_approx_eq!(1.0, 1.0);
        assert_approx_eq!(1.0, 1.0 + f32::EPSILON / 2.0);
        assert_approx_eq!(1.0, 1.0 - f32::EPSILON / 2.0);
    }

    #[test]
    fn f64_approx_eq() {
        assert_approx_eq!(1.0, 1.0);
        assert_approx_eq!(1.0, 1.0 + f64::EPSILON / 2.0);
        assert_approx_eq!(1.0, 1.0 - f64::EPSILON / 2.0);
    }

    #[test]
    fn vec2_approx_eq() {
        use bevy::math::Vec2;

        assert_approx_eq!(Vec2::new(1.0, 2.0), Vec2::new(1.0, 2.0));
        assert_approx_eq!(
            Vec2::new(1.0, 2.0),
            Vec2::new(1.0 + f32::EPSILON / 2.0, 2.0)
        );
        assert_approx_eq!(
            Vec2::new(1.0, 2.0),
            Vec2::new(1.0, 2.0 + f32::EPSILON / 2.0)
        );
    }

    #[test]
    fn vec3_approx_eq() {
        use bevy::math::Vec3;

        assert_approx_eq!(Vec3::new(1.0, 2.0, 3.0), Vec3::new(1.0, 2.0, 3.0));
        assert_approx_eq!(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(1.0 + f32::EPSILON / 2.0, 2.0, 3.0)
        );
        assert_approx_eq!(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(1.0, 2.0 + f32::EPSILON / 2.0, 3.0)
        );
        assert_approx_eq!(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(1.0, 2.0, 3.0 + f32::EPSILON / 2.0)
        );
    }
}
