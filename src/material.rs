//! Module containing [`Material`] trait, and implementors of it, such as:
//! * [`Lambertian`]
//! * [`Metal`]

use crate::color::Color;
use crate::ray::HitRecord;
use crate::ray::Ray;
use crate::vector::Vector;

pub enum EmergentRayInteraction<'a> {
    Absorbed,
    Scattered(EmergentRay<'a>),
}

pub struct EmergentRay<'a> {
    pub(crate) inner: Ray<'a>,
    pub(crate) attenuation: Color,
}

// Could actually make `Material` an enum. It's possible I think.
pub trait Material: Send + Sync {
    fn interact<'a>(&self, ray: &Ray, record: &'a HitRecord) -> EmergentRayInteraction<'a>;
}

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
    fn interact<'a>(&self, _ray: &Ray, record: &'a HitRecord) -> EmergentRayInteraction<'a> {
        // Non-Lambertian implementation:
        // let direction = &record.normal + &Vector::random_on_hemisphere(&record.normal);

        let scatter_direction = &record.normal + &Vector::random_unit();
        let scattered_ray = Ray::new(&record.point, scatter_direction);
        EmergentRayInteraction::Scattered(EmergentRay {
            attenuation: self.albedo,
            inner: scattered_ray,
        })
    }
}

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
    fn interact<'a>(&self, ray: &Ray, record: &'a HitRecord) -> EmergentRayInteraction<'a> {
        let reflected_direction =
            ray.dir().reflect(&record.normal).unit() + (Vector::random_unit() * self.roughness);
        if reflected_direction.dot(&record.normal) < 0.0 {
            return EmergentRayInteraction::Absorbed;
        }
        let reflected_ray = Ray::new(&record.point, reflected_direction);
        EmergentRayInteraction::Scattered(EmergentRay {
            attenuation: self.albedo,
            inner: reflected_ray,
        })
    }
}
