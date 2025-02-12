//! Module defining the [`Ray`] struct and the [`Hittable`] trait

use std::rc::Rc;

use crate::color::Color;
use crate::utils::interval::Interval;
use crate::vector::{Point, Vector};

use thiserror::Error;

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

    pub fn color(&self, world: &HittableList) -> Color {
        match world.hit(Interval::new(0.0, f64::MAX), self) {
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
    pub(super) front_face: bool,
}

impl HitRecord {
    pub fn face_normal(&mut self, ray: &Ray, outward_normal: &Vector<f64, 3>) -> Result<(), Error> {
        if outward_normal.norm_squared() != 1.0 {
            return Err(Error::NonNormalVector);
        }
        self.front_face = ray.dir().dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal.clone()
        } else {
            outward_normal.clone() * -1.0
        };
        Ok(())
    }
}

#[derive(Clone)]
pub struct HittableList(Vec<Rc<dyn Hittable>>);

impl HittableList {
    /// Create a new [`HittableList`]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Add a [`Hittable`] object to a [`HittableList`]. Currently never returns an error, even though it is currently typed not as such.
    ///
    /// ```rs
    /// let mut world = HittableList::new();
    /// let sphere = Sphere::new(Point::new(0.0, 0.0, -1.0), 0.5);
    /// let ground = Sphere::new(Point::new(0.0, -100.5, -1.0), 100.0);
    /// world.add(Rc::new(sphere))?;
    /// world.add(Rc::new(ground))?;
    /// ```
    pub fn add(&mut self, object: Rc<dyn Hittable>) -> Result<(), Error> {
        self.0.push(Rc::clone(&object));
        Ok(())
    }
}

// Treat HittableList like a "world" object: a composition of [`Hittable`]s. Every object in [`HittableList`] is [`Hittable`], so [`HittableList`] is hittable.
impl Hittable for HittableList {
    /// Loops through every [`Hittable`] in the underlying [`Vec<Rc<dyn Hittable>>`]
    fn hit(&self, ray_t: Interval, ray: &Ray) -> Option<HitRecord> {
        let mut hit_record = None;
        // Never hit
        let mut closest_so_far = ray_t.max;
        for hittable in &self.0 {
            if let Some(rec) = hittable.hit(Interval::new(ray_t.min, closest_so_far), ray) {
                // This hit will be (should be; really depending on the implementor of `Hittable`) closer
                closest_so_far = rec.t;
                hit_record = Some(rec);
            }
        }
        hit_record
    }
}

pub trait Hittable {
    fn hit(&self, ray_t: Interval, ray: &Ray) -> Option<HitRecord>;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Vector is expected to be normal")]
    NonNormalVector,
}
