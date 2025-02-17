use crate::utils::rand::random;

use super::EmergentRay;
use super::Material;
use super::RayInteraction;

use crate::color::Color;
use crate::objects::HitRecord;
use crate::ray::Ray;
use crate::vector::Vector;

pub struct Dielectric {
    ior: f64,
}

impl Dielectric {
    pub fn new(ior: f64) -> Self {
        Self { ior }
    }

    fn reflectance(cosine: f64, ior: f64) -> f64 {
        // Shlick's approximation for reflectance
        let mut r0 = (1.0 - ior) / (1.0 + ior);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn interact<'a>(&self, ray: &Ray, record: &'a HitRecord) -> RayInteraction<'a> {
        let ior = if record.front_face {
            1.0 / self.ior
        } else {
            self.ior
        };

        let incident = ray.dir();

        let cos_theta = (-incident).dot(record.normal.inner()).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let direction = if ior * sin_theta > 1.0 || Self::reflectance(cos_theta, ior) > random() {
            // TIR
            incident.reflect(&record.normal)
        } else {
            incident.refract(&record.normal, ior)
        };

        RayInteraction::Scattered(EmergentRay {
            inner: Ray::new(&record.point, direction),
            attenuation: Color::new(1.0, 1.0, 1.0),
        })
    }
}
