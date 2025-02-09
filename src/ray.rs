//! Module for the [`Ray`] struct

use crate::color::Color;
use crate::vector::{Point, Vector};

pub struct Ray<'a> {
    origin: &'a Point<f64>,
    dir: &'a Vector<f64, 3>,
}

impl<'a> Ray<'a> {
    /// Creates a new [`Ray`].
    #[inline]
    pub fn new(origin: &'a Point<f64>, dir: &'a Vector<f64, 3>) -> Self {
        Self { origin, dir }
    }

    pub fn at(&self, t: f64) -> Point<f64> {
        self.origin + &(self.dir * t)
    }

    pub fn origin(&self) -> &Point<f64> {
        &self.origin
    }

    pub fn dir(&self) -> &Vector<f64, 3> {
        &self.dir
    }

    pub fn color(&self) -> Color {
        let unit_direction = self.dir().unit();
        let a = (unit_direction.y() + 1.0) * 0.5;
        Color::new(255, 255, 255) * (1.0 - a) + Color::new(128, 179, 255) * a
    }
}
