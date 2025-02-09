use crate::ray::{HitRecord, Hittable, Ray};
use crate::vector::Point;

pub struct Sphere {
    center: Point<f64>,
    radius: f64,
}

impl Sphere {
    pub fn new(center: Point<f64>, radius: f64) -> Self {
        Self { center, radius }
    }
}

impl Hittable for Sphere {
    fn hit(&self, t_min: f64, t_max: f64, ray: &Ray) -> Option<HitRecord> {
        let oc = &self.center - ray.origin();
        let a = ray.dir().norm_squared();
        let h = oc.dot(ray.dir());
        let c = oc.norm_squared() - self.radius.powi(2);
        let discrim = h.powi(2) - a * c;
        if discrim < 0.0 {
            return None;
        }
        let t = (h - discrim.sqrt()) / a;
        if t < t_min || t > t_max {
            let t = (h + discrim.sqrt()) / a;
            if t < t_min || t > t_max {
                return None;
            }
        }
        Some(HitRecord {
            t,
            point: ray.at(t),
            normal: (&ray.at(t) - &self.center) / self.radius,
        })
    }
}
