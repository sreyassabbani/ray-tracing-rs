//! Module defining the [`Ray`] struct and the [`Hittable`] trait

use crate::color::Color;
use crate::objects::Sphere;
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
        let center = Point::new(0.0, 0.0, -1.0);
        let sphere = Sphere::new(center.clone(), 0.5);
        match sphere.hit(0.0, 10000.0, self) {
            Some(record) => {
                let normal = record.normal;
                return Color::new(
                    ((normal.x() + 1.0) * 127.5) as u8,
                    ((normal.y() + 1.0) * 127.5) as u8,
                    ((normal.z() + 1.0) * 127.5) as u8,
                );
            }
            None => {
                let unit_direction = self.dir().unit();
                let a = (unit_direction.y() + 1.0) * 0.5;
                let b = Color::new(128, 179, 255);
                Color::new(255, 255, 255) * (1.0 - a) + b * a
            }
        }
    }
}

pub struct HitRecord {
    pub(super) point: Point<f64>,
    pub(super) normal: Vector<f64, 3>,
    pub(super) t: f64,
}

pub trait Hittable {
    fn hit(&self, t_min: f64, t_max: f64, ray: &Ray) -> Option<HitRecord>;
}
