use super::EmergentRay;
use super::Material;
use super::RayInteraction;

use crate::color::Color;
use crate::objects::HitRecord;
use crate::ray::Ray;
use crate::vector::Vector;

#[derive(Clone)]
pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn interact<'a>(&self, _ray: &Ray, record: &'a HitRecord) -> RayInteraction<'a> {
        // Non-Lambertian implementation:
        // let direction = &record.normal + &Vector::random_on_hemisphere(&record.normal);

        let scatter_direction = (record.normal.inner() + &Vector::random_unit()).unit();
        let scattered_ray = Ray::new(&record.point, scatter_direction);
        RayInteraction::Scattered(EmergentRay {
            attenuation: self.albedo,
            inner: scattered_ray,
        })
    }
}
