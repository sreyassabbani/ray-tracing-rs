//! Module defining objects to be held inside of a world, which is of type [`HittableList`]
//!
//! Contains
//! * [`Sphere`]

#![warn(missing_docs)]

pub mod sphere;

pub use sphere::Sphere;

use std::sync::Arc;

use thiserror::Error;

use crate::materials::Material;
use crate::ray::Ray;
use crate::utils::interval::Interval;
use crate::vector::{Point, UtVector};

pub struct HitRecord {
    pub(super) point: Point,
    pub(super) normal: UtVector,
    pub(super) t: f64,
    pub(super) front_face: bool,
    // Could this possibly be reduced down to `Box`? Look into various implementations of `Hittable` trait for objects
    pub(super) material: Arc<dyn Material>,
}

impl HitRecord {
    pub fn face_normal(&mut self, ray: &Ray, outward_normal: &UtVector) {
        self.front_face = ray.dir_v().dot(outward_normal) < 0.0;

        self.normal = if self.front_face {
            outward_normal.clone()
        } else {
            -outward_normal
        };
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
    pub fn add(&mut self, object: impl Hittable + 'static) -> Result<&mut Self, Error> {
        self.0.push(Arc::new(object));
        Ok(self)
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
pub enum Error {}
