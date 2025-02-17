//! Module defining the [`Ray`] struct and the [`Hittable`] trait
//!
//! Closely related to [`crate::material`] module. That module exports the type [`EmergentRay`] (design decisions on this might need to be reviewed).

use crate::color::Color;
use crate::materials::{Material, RayInteraction};
use crate::objects::{Hittable, HittableList};
use crate::utils::interval::Interval;
use crate::vector::{Point, UtVector, Vector};

/// A struct for representing rays
///
// Could shared `origin` be premature optimization? Maybe.
#[derive(Clone, Copy, Debug)]
pub struct Ray<'o> {
    origin: &'o Point,
    dir: UtVector,
}

impl<'o> Ray<'o> {
    /// Creates a new [`Ray`].
    pub fn new(origin: &'o Point, dir: UtVector) -> Self {
        Self { origin, dir }
    }

    pub fn origin(&self) -> &Point {
        self.origin
    }

    pub fn dir(&self) -> &UtVector {
        &self.dir
    }

    pub fn dir_v(&self) -> &Vector {
        self.dir.inner()
    }

    pub fn at(&self, t: f64) -> Point {
        self.origin + &(self.dir.inner() * t)
    }

    pub fn color(&self, world: &HittableList, bounce: u32) -> Color {
        // Limit the number of child rays
        if bounce == 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        // Use 0.001 instead of 0.0 to avoid shadow acne
        match world.hit(Interval::new(0.001, f64::MAX), self) {
            Some(record) => {
                use RayInteraction::*;
                // Self interacts with material, and send in corresponding record of its interaction (awkward)
                match record.material.interact(self, &record) {
                    Absorbed => {
                        return Color::new(0.0, 0.0, 0.0);
                    }
                    Scattered(emergent_ray) => {
                        return emergent_ray.attenuation
                            * emergent_ray.inner.color(world, bounce - 1)
                    }
                }
            }
            // Render the sky instead
            None => {
                let unit_direction = self.dir_v().unit();
                let a = (unit_direction.y() + 1.0) * 0.5;
                let b = Color::new(0.5, 0.70196, 1.0);
                Color::new(1.0, 1.0, 1.0) * (1.0 - a) + b * a
            }
        }
    }
}
