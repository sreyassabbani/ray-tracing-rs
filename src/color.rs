//! Module containing [`Color`] and logic for operations with it.

use std::fmt;
use std::ops;

#[derive(Clone)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Color { r, g, b }
    }
}

impl ops::Mul<f64> for Color {
    type Output = Color;
    fn mul(self, rhs: f64) -> Self::Output {
        Color::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl ops::Add<f64> for Color {
    type Output = Color;
    fn add(self, rhs: f64) -> Self::Output {
        Color::new(self.r + rhs, self.g + rhs, self.b + rhs)
    }
}

impl ops::Div<f64> for Color {
    type Output = Color;
    fn div(self, rhs: f64) -> Self::Output {
        Color::new(self.r / rhs, self.g / rhs, self.b / rhs)
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Self::Output {
        Color::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl ops::AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // P3 PPM format
        let r = (255.0 * self.r.clamp(0.0, 1.0)) as u8;
        let g = (255.0 * self.g.clamp(0.0, 1.0)) as u8;
        let b = (255.0 * self.b.clamp(0.0, 1.0)) as u8;

        write!(f, "{} {} {}", r, g, b)
    }
}
