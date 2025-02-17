use std::sync::Arc;

use super::{HitRecord, Hittable};
use crate::materials::Material;
use crate::ray::Ray;
use crate::utils::interval::Interval;
use crate::vector::Point;

pub struct Sphere {
    center: Point,
    radius: f64,
    material: Arc<dyn Material>,
}

impl Sphere {
    // Should I take `Arc<Box<dyn Material>>` instead? Little worried about `'static` and maybe there's a way around not enforcing static lifetime of the material.
    // See other ways if possible
    pub fn new(center: Point, radius: f64, material: impl Material + 'static) -> Self {
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
        let a = ray.dir_v().len_squared();
        let h = oc.dot(ray.dir_v());
        let c = oc.len_squared() - self.radius.powi(2);
        let discrim = h.powi(2) - a * c;

        if discrim < 0.0 {
            return None;
        }

        let mut t = (h - discrim.sqrt()) / a;
        if t < ray_t.min || t > ray_t.max {
            t = (h + discrim.sqrt()) / a;
            if t < ray_t.min || t > ray_t.max {
                return None;
            }
        }

        // Even though the vector seems to emanate from the center of the circle, it is still a normal vector to the sphere's surface. Keep that in mind. Also, we divide by `radius` because of negative-radii spheres apparently instead of normalizing by length.
        let mut normal = ((&ray.at(t) - &self.center) / self.radius).is_unit_unsafe();

        let front_face = ray.dir_v().dot(&normal) < 0.0;
        if !front_face {
            normal = -normal;
        }
        Some(HitRecord {
            t,
            point: ray.at(t),
            front_face,
            normal,
            material: Arc::clone(&self.material),
        })
    }
}
