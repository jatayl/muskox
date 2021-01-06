use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub};

use ordered_float::OrderedFloat;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Score {
    data: OrderedFloat<f32>, // maybe make this a tuple instead idk
}

impl Score {
    pub const INFINITY: Score = Score {
        data: OrderedFloat(f32::INFINITY),
    };

    pub const NEG_INFINITY: Score = Score {
        data: OrderedFloat(f32::NEG_INFINITY),
    };
}

impl From<f32> for Score {
    fn from(t: f32) -> Score {
        Score {
            data: OrderedFloat(t),
        }
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.cmp(&other.data)
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for Score {}

impl Add for Score {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Score {
            data: self.data + other.data,
        }
    }
}

impl Sub for Score {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Score {
            data: self.data - other.data,
        }
    }
}

impl Mul for Score {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Score {
            data: self.data * other.data,
        }
    }
}

impl Div for Score {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        Score {
            data: self.data / other.data,
        }
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, other: Self) {
        *self = Score {
            data: self.data + other.data,
        }
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.fmt(f)
    }
}
