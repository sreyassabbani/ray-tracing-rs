//! Module containing [`Material`] trait, and implementors of it, such as:
//! * [`Lambertian`]
//! * [`Metal`]

pub mod dielectric;
pub mod lambertian;
pub mod metal;

pub use dielectric::Dielectric;
pub use lambertian::Lambertian;
pub use metal::Metal;

use crate::color::Color;
use crate::objects::HitRecord;
use crate::ray::Ray;

pub enum RayInteraction<'a> {
    Absorbed,
    Scattered(EmergentRay<'a>),
}

pub struct EmergentRay<'a> {
    pub(crate) inner: Ray<'a>,
    pub(crate) attenuation: Color,
}

pub trait Material: Send + Sync {
    fn interact<'a>(&self, ray: &Ray, record: &'a HitRecord) -> RayInteraction<'a>;
}
