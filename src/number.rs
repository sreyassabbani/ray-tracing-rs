//! Module defining numeric traits

use std::{fmt, ops};

// NOTE: look into monoids
pub trait AdditiveIdentity {
    fn additive_identity() -> Self;
}

pub trait MultiplicativeIdentity {
    fn multiplicative_identity() -> Self;
}

pub trait Numeric<T>:
    Default
    + Copy
    + AdditiveIdentity
    + MultiplicativeIdentity
    + ops::Add<Output = T>
    + ops::AddAssign
    + ops::Sub<Output = T>
    + ops::SubAssign
    + ops::Mul<Output = T>
    + ops::MulAssign
    + ops::Div<Output = T>
    + ops::DivAssign
    + Into<f64>
    + From<f64>
    + fmt::Debug
{
}

impl AdditiveIdentity for f64 {
    fn additive_identity() -> Self {
        0.0
    }
}

impl MultiplicativeIdentity for f64 {
    fn multiplicative_identity() -> Self {
        1.0
    }
}

impl Numeric<f64> for f64 {}
