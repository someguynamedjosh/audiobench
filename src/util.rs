use num::Float;

pub trait FloatUtil: Sized + Copy {
    /// Converts from the range [from_min,from_max] to [0,1]
    fn from_range(self, from_min: Self, from_max: Self) -> Self;
    /// Converts from the range [0,1] to [to_min, to_max]
    fn to_range(self, to_min: Self, to_max: Self) -> Self;
    fn from_range_to_range(
        self,
        from_min: Self,
        from_max: Self,
        to_min: Self,
        to_max: Self,
    ) -> Self {
        self.from_range(from_min, from_max).to_range(to_min, to_max)
    }

    /// Clamps the value to the specified range. Not called clamp because that causes an error.
    fn clam(self, min: Self, max: Self) -> Self;
}

impl<T: Float> FloatUtil for T {
    fn from_range(self, from_min: Self, from_max: Self) -> Self {
        (self - from_min) / (from_max - from_min)
    }

    fn to_range(self, to_min: Self, to_max: Self) -> Self {
        self * (to_max - to_min) + to_min
    }

    fn clam(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

pub fn format_decimal(value: f32, digits: i32) -> String {
    let digits = match value {
        v if v <= 0.0 => digits,
        _ => digits - (value.abs().log10().min(digits as f32 - 1.0) as i32),
    };
    let digits = digits as usize;
    format!("{:.*}", digits, value)
}

pub use std::{cell::RefCell, rc::Rc, sync::Arc};

pub type Rcrc<T> = Rc<RefCell<T>>;
pub fn rcrc<T>(content: T) -> Rcrc<T> {
    Rc::new(RefCell::new(content))
}
