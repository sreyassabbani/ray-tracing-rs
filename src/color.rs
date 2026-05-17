//! Module containing [`Color`] and logic for operations with it.

use std::fmt;
use std::ops;

use crate::utils::rand::{random, random_range};
// Was `Copy` a good idea?
#[derive(Clone, Copy, Debug)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Color { r, g, b }
    }

    pub fn random() -> Self {
        Color {
            r: random(),
            g: random(),
            b: random(),
        }
    }

    pub fn random_range(min: f64, max: f64) -> Self {
        Color {
            r: random_range(min, max),
            g: random_range(min, max),
            b: random_range(min, max),
        }
    }

    /// Convert this linear RGB color to gamma-corrected 8-bit RGB.
    pub fn to_rgb8(self) -> [u8; 3] {
        [
            linear_channel_to_display_u8(self.r),
            linear_channel_to_display_u8(self.g),
            linear_channel_to_display_u8(self.b),
        ]
    }

    /// Convert this linear RGB color to gamma-corrected 8-bit RGBA.
    pub fn to_rgba8(self) -> [u8; 4] {
        let [r, g, b] = self.to_rgb8();
        [r, g, b, 255]
    }
}

fn linear_channel_to_display_u8(channel: f64) -> u8 {
    let gamma_corrected = if channel > 0.0 { channel.sqrt() } else { 0.0 };
    (255.0 * gamma_corrected.clamp(0.0, 1.0)) as u8
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
        let [r, g, b] = self.to_rgb8();

        write!(f, "{} {} {}", r, g, b)
    }
}
