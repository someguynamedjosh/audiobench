use num::Float;

pub trait RangeMap: Sized + Copy {
    /// Converts from the range [from_min,from_max] to [0,1]
    fn from_range(self, from_min: Self, from_max: Self) -> Self;
    /// Converts from the range [0,1] to [to_min, to_max]
    fn to_range(self, to_min: Self, to_max: Self) -> Self;
    fn from_range_to_range(self, from_min: Self, from_max: Self, to_min: Self, to_max: Self) -> Self {
        self.from_range(from_min, from_max).to_range(to_min, to_max)
    }
}

impl<T: Float> RangeMap for T {
    fn from_range(self, from_min: Self, from_max: Self) -> Self {
        (self - from_min) / (from_max - from_min)
    }

    fn to_range(self, to_min: Self, to_max: Self) -> Self {
        self * (to_max - to_min) + to_min
    }
}
