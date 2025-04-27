use std::sync::Arc;

use super::{HitRecord, Hittable};
use crate::materials::Material;
use crate::ray::Ray;
use crate::utils::interval::Interval;
use crate::vector::UtVector;

/// Object representing a plane in three-dimensions.
pub struct Plane {
    normal: UtVector,
    d: f64,
    material: Arc<dyn Material>,
}

impl Plane {
    // Should I take `Arc<Box<dyn Material>>` instead? Little worried about `'static` and maybe there's a way around not enforcing static lifetime of the material.
    // See other ways if possible
    /// Create a new [`Plane`] with a `normal` and `d` offset.
    pub fn new(normal: UtVector, d: f64, material: impl Material + 'static) -> Self {
        Self {
            normal,
            d,
            material: Arc::new(material),
        }
    }
}

impl Hittable for Plane {
    fn hit(&self, ray_t: Interval, ray: &Ray) -> Option<HitRecord> {
        let denom = self.normal.dot(ray.dir());

        if denom.abs() < 0.001 {
            return None;
        }

        let t = -(self.normal.dot(ray.origin()) + self.d) / denom;

        if !ray_t.contains(t) {
            return None;
        }

        let point = ray.origin() + ray.dir() * t;

        let front_face = ray.dir().dot(&self.normal) < 0.0;
        let outward_normal = if front_face {
            self.normal
        } else {
            -self.normal
        };

        Some(HitRecord {
            point,
            normal: outward_normal,
            t,
            front_face,
            material: Arc::clone(&self.material),
        })
    }
}

// Maybe generalize this formula
