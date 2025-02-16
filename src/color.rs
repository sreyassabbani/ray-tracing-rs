//! Module containing [`Color`] and logic for operations with it.

use std::fmt;
use std::ops;

// Was `Copy` a good idea?
#[derive(Clone, Copy)]
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

impl ops::Mul<Color> for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
    }
}

impl ops::MulAssign<f64> for Color {
    fn mul_assign(&mut self, rhs: f64) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
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
        // Pray compiler optimizes this
        let linear_to_gamma = |e: f64| if e > 0.0 { e.sqrt() } else { 0.0 };

        let r = linear_to_gamma(self.r);
        let g = linear_to_gamma(self.g);
        let b = linear_to_gamma(self.b);

        // P3 PPM format
        let r = (255.0 * r.clamp(0.0, 1.0)) as u8;
        let g = (255.0 * g.clamp(0.0, 1.0)) as u8;
        let b = (255.0 * b.clamp(0.0, 1.0)) as u8;

        write!(f, "{} {} {}", r, g, b)
    }
}
