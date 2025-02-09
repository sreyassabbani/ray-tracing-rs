use std::ops;

#[derive(Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

impl ops::Mul<u8> for Color {
    type Output = Color;
    fn mul(self, rhs: u8) -> Self::Output {
        Color::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl ops::Add<u8> for Color {
    type Output = Color;
    fn add(self, rhs: u8) -> Self::Output {
        Color::new(self.r + rhs, self.g + rhs, self.b + rhs)
    }
}
