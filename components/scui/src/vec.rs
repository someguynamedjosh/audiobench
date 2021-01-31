use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

impl Vec2D {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn new_int(x: i32, y: i32) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Returns the angle in radians according to atan2()
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Returns length of this vector using pythagorean distance.
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn inside(self, other: Self) -> bool {
        self.x >= 0.0 && self.y >= 0.0 && self.x <= other.x && self.y <= other.y
    }

    pub fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

impl Neg for Vec2D {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T: Into<Vec2D>> Add<T> for Vec2D {
    type Output = Self;

    fn add(self: Self, other: T) -> Self {
        let other = other.into();
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Into<Vec2D>> AddAssign<T> for Vec2D {
    fn add_assign(&mut self, other: T) {
        let other = other.into();
        self.x += other.x;
        self.y += other.y;
    }
}

impl<T: Into<Vec2D>> Sub<T> for Vec2D {
    type Output = Self;

    fn sub(self: Self, other: T) -> Self {
        let other = other.into();
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Into<Vec2D>> SubAssign<T> for Vec2D {
    fn sub_assign(&mut self, other: T) {
        let other = other.into();
        self.x -= other.x;
        self.y += other.y;
    }
}

impl<T: Into<Vec2D>> Mul<T> for Vec2D {
    type Output = Self;

    fn mul(self: Self, other: T) -> Self {
        let other = other.into();
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<T: Into<Vec2D>> MulAssign<T> for Vec2D {
    fn mul_assign(&mut self, other: T) {
        let other = other.into();
        self.x *= other.x;
        self.y *= other.y;
    }
}

impl<T: Into<Vec2D>> Div<T> for Vec2D {
    type Output = Self;

    fn div(self: Self, other: T) -> Self {
        let other = other.into();
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl<T: Into<Vec2D>> DivAssign<T> for Vec2D {
    fn div_assign(&mut self, other: T) {
        let other = other.into();
        self.x /= other.x;
        self.y /= other.y;
    }
}

impl From<f32> for Vec2D {
    fn from(other: f32) -> Vec2D {
        Self { x: other, y: other }
    }
}

impl From<(f32, f32)> for Vec2D {
    fn from(other: (f32, f32)) -> Vec2D {
        Self {
            x: other.0,
            y: other.1,
        }
    }
}

impl From<i32> for Vec2D {
    fn from(other: i32) -> Vec2D {
        Self {
            x: other as f32,
            y: other as f32,
        }
    }
}

impl From<(i32, i32)> for Vec2D {
    fn from(other: (i32, i32)) -> Vec2D {
        Self {
            x: other.0 as f32,
            y: other.1 as f32,
        }
    }
}
