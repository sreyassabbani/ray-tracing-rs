use super::EmergentRay;
use super::Material;
use super::RayInteraction;

use crate::color::Color;
use crate::objects::HitRecord;
use crate::ray::Ray;
use crate::vector::Vector;

#[derive(Clone)]
pub struct Metal {
    albedo: Color,
    roughness: f64,
}

impl Metal {
    pub fn new(albedo: Color, roughness: f64) -> Self {
        Self { albedo, roughness }
    }
}

impl Material for Metal {
    fn interact<'a>(&self, ray: &Ray, record: &'a HitRecord) -> RayInteraction<'a> {
        let reflected_direction = (ray.dir().reflect(&record.normal).unit()
            + (Vector::random_unit() * self.roughness))
            .unit();
        if reflected_direction.dot(&record.normal) < 0.0 {
            return RayInteraction::Absorbed;
        }
        let reflected_ray = Ray::new(&record.point, reflected_direction);
        RayInteraction::Scattered(EmergentRay {
            attenuation: self.albedo,
            inner: reflected_ray,
        })
    }
}
