//! Module defining objects to be held inside of a world, which is of type [`HittableList`]
//!
//! Contains
//! * [`Sphere`]

use std::sync::Arc;

use crate::material::Material;
use crate::ray::{HitRecord, Hittable, Ray};
use crate::utils::interval::Interval;
use crate::vector::Point;

pub struct Sphere {
    center: Point<f64>,
    radius: f64,
    material: Arc<dyn Material>,
}

impl Sphere {
    // Should I take `Arc<Box<dyn Material>>` instead? Little worried about `'static` and maybe there's a way around not enforcing static lifetime of the material.
    // See other ways if possible
    pub fn new(center: Point<f64>, radius: f64, material: impl Material + 'static) -> Self {
        Self {
            center,
            radius,
            material: Arc::new(material),
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray_t: Interval, ray: &Ray) -> Option<HitRecord> {
        let oc = &self.center - ray.origin();
        let a = ray.dir().norm_squared();
        let h = oc.dot(ray.dir());
        let c = oc.norm_squared() - self.radius.powi(2);
        let discrim = h.powi(2) - a * c;
        if discrim < 0.0 {
            return None;
        }
        let t = (h - discrim.sqrt()) / a;
        if !ray_t.contains_inclusive(t) {
            let t = (h + discrim.sqrt()) / a;
            if !ray_t.contains_inclusive(t) {
                return None;
            }
        }

        // Even though the vector seems to emanate from the center of the circle, it is still a normal vector to the sphere's surface. Keep that in mind.
        let normal = (&ray.at(t) - &self.center) / self.radius;
        Some(HitRecord {
            t,
            point: ray.at(t),
            front_face: ray.dir().dot(&normal) > 0.0,
            // Move normal into [`HitRecord`]
            normal,
            material: Arc::clone(&self.material),
        })
    }
}
