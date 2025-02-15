//! Module defining the [`Ray`] struct and the [`Hittable`] trait

use std::sync::Arc;

use crate::color::Color;
use crate::utils::interval::Interval;
use crate::vector::{Point, Vector};

use thiserror::Error;

/// A struct for representing rays
///
/// Note: `origin` is a reference, whereas `dir` is an owned value. This is a very intentional design decision (that could be bad).
#[derive(Clone)]
pub struct Ray<'a> {
    origin: &'a Point<f64>,
    dir: Vector<f64, 3>,
}

impl<'a> Ray<'a> {
    /// Creates a new [`Ray`].
    #[inline]
    pub fn new(origin: &'a Point<f64>, dir: Vector<f64, 3>) -> Self {
        Self { origin, dir }
    }

    #[inline]
    pub fn at(&self, t: f64) -> Point<f64> {
        self.origin + &(&self.dir * t)
    }

    #[inline]
    pub fn origin(&self) -> &Point<f64> {
        self.origin
    }

    #[inline]
    pub fn dir(&self) -> &Vector<f64, 3> {
        &self.dir
    }

    pub fn color(&self, world: &HittableList, bounce: u32) -> Color {
        // Limit the number of child rays
        if bounce == 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        // Use 0.001 instead of 0.0 to avoid shadow acne
        match world.hit(Interval::new(0.001, f64::MAX), self) {
            Some(record) => {
                // Naive implementation:
                // let direction = &record.normal + &Vector::random_on_hemisphere(&record.normal);

                // Lambertian reflection
                let direction = &record.normal + &Vector::random_unit();

                let scattered_ray = Ray::new(&record.point, direction);
                return scattered_ray.color(world, bounce - 1) * 0.5;
            }
            // Render the sky instead
            None => {
                let unit_direction = self.dir().unit();
                let a = (unit_direction.y() + 1.0) * 0.5;
                let b = Color::new(0.5, 0.70196, 1.0);
                Color::new(1.0, 1.0, 1.0) * (1.0 - a) + b * a
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
pub struct HittableList(Vec<Arc<dyn Hittable>>);

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
    pub fn add(&mut self, object: Arc<dyn Hittable>) -> Result<(), Error> {
        self.0.push(Arc::clone(&object));
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

pub trait Hittable: Send + Sync {
    fn hit(&self, ray_t: Interval, ray: &Ray) -> Option<HitRecord>;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Vector is expected to be normal")]
    NonNormalVector,
}
