//! Module defining [`Vector`], and variants [`Point`], [`UtVector`]

use crate::utils::rand;

use std::{
    array,
    ops::{self, Deref},
};

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    x: f64,
    y: f64,
    z: f64,
}

impl From<[f64; 3]> for Vector {
    fn from(value: [f64; 3]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl Vector {
    /// Create a new [`Vector`]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        Vector::from([
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        ])
    }

    pub fn len_squared(&self) -> f64 {
        self.dot(self)
    }

    pub fn len(&self) -> f64 {
        self.len_squared().sqrt()
    }

    pub fn unit(&self) -> UtVector {
        UtVector {
            v: self / self.len(),
        }
    }

    pub fn is_unit(self) -> Result<UtVector, Error> {
        if self.len_squared() != 1.0 {
            return Err(Error::NotUnitVector);
        }
        Ok(UtVector { v: self })
    }

    pub fn assert_unit_unsafe(self) -> UtVector {
        UtVector { v: self }
    }

    pub fn random() -> Self {
        // Generate a different random number for each component
        Vector::from(array::from_fn(|_| rand::random()))
    }

    pub fn random_range(min: f64, max: f64) -> Self {
        // Generate a different random number for each component
        Vector::from(array::from_fn(|_| rand::random_range(min, max)))
    }

    pub fn random_unit() -> Self {
        loop {
            let p = Self::random_range(-1.0, 1.0);
            let nsq = p.len();
            if 1e-160 < nsq && nsq <= 1.0 {
                return p / nsq;
            }
        }
    }

    pub fn random_on_hemisphere(normal: &Self) -> Self {
        let on_unit_sphere = Self::random_unit();
        if on_unit_sphere.dot(normal) > 0.0 {
            return on_unit_sphere;
        } else {
            return on_unit_sphere * -1.0;
        }
    }
}

mod _utils {
    //! Internal helper module for defining operations on [`Vector`]

    use super::*;

    pub(super) fn add_vectors(lhs: &Vector, rhs: &Vector) -> Vector {
        Vector::from([lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z])
    }

    pub(super) fn sub_vectors(lhs: &Vector, rhs: &Vector) -> Vector {
        Vector::from([lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z])
    }

    pub(super) fn mul_vector_and_scalar(lhs: &Vector, rhs: f64) -> Vector {
        Vector::from([lhs.x * rhs, lhs.y * rhs, lhs.z * rhs])
    }

    pub(super) fn div_vector_and_scalar(lhs: &Vector, rhs: f64) -> Vector {
        Vector::from([lhs.x / rhs, lhs.y / rhs, lhs.z / rhs])
    }

    pub(super) fn add_assign_num_to_vector(value: &mut Vector, rhs: f64) {
        value.x += rhs;
        value.y += rhs;
        value.z += rhs;
    }

    pub(super) fn sub_assign_num_to_vector(value: &mut Vector, rhs: f64) {
        value.x -= rhs;
        value.y -= rhs;
        value.z -= rhs;
    }

    pub(super) fn mul_assign_num_to_vector(value: &mut Vector, rhs: f64) {
        value.x *= rhs;
        value.y *= rhs;
        value.z *= rhs;
    }

    pub(super) fn div_assign_num_to_vector(value: &mut Vector, rhs: f64) {
        value.x /= rhs;
        value.y /= rhs;
        value.z /= rhs;
    }

    pub(super) fn neg_vector(value: &Vector) -> Vector {
        Vector::from([-value.x, -value.y, -value.z])
    }

    pub(super) fn neg_utvector(value: &UtVector) -> UtVector {
        UtVector {
            v: Vector::from([-value.x, -value.y, -value.z]),
        }
    }

    pub(super) fn add_vector_to_utvector(lhs: &Vector, rhs: &UtVector) -> Vector {
        Vector::from([lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z])
    }

    pub(super) fn sub_vector_to_utvector(lhs: &Vector, rhs: &UtVector) -> Vector {
        Vector::from([lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z])
    }

    pub(super) fn mul_vector_to_utvector(lhs: &Vector, rhs: &UtVector) -> Vector {
        Vector::from([lhs.x * rhs.x, lhs.y * rhs.y, lhs.z * rhs.z])
    }

    pub(super) fn div_vector_to_utvector(lhs: &Vector, rhs: &UtVector) -> Vector {
        Vector::from([lhs.x / rhs.x, lhs.y / rhs.y, lhs.z / rhs.z])
    }
}

use _utils::*;
use thiserror::Error;

// Addition implementations
impl ops::Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        add_vectors(&self, &rhs)
    }
}

impl ops::Add<Vector> for &Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        add_vectors(self, &rhs)
    }
}

impl ops::Add<&Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: &Vector) -> Self::Output {
        add_vectors(&self, rhs)
    }
}

impl ops::Add<&Vector> for &Vector {
    type Output = Vector;

    fn add(self, rhs: &Vector) -> Self::Output {
        add_vectors(&self, &rhs)
    }
}

impl ops::AddAssign<f64> for Vector {
    fn add_assign(&mut self, rhs: f64) {
        add_assign_num_to_vector(self, rhs)
    }
}

// Subtraction implementations
impl ops::Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        sub_vectors(&self, &rhs)
    }
}

impl ops::Sub<Vector> for &Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        sub_vectors(self, &rhs)
    }
}

impl ops::Sub<&Vector> for Vector {
    type Output = Vector;

    fn sub(self, rhs: &Vector) -> Self::Output {
        sub_vectors(&self, rhs)
    }
}

impl ops::Sub<&Vector> for &Vector {
    type Output = Vector;

    fn sub(self, rhs: &Vector) -> Self::Output {
        sub_vectors(&self, &rhs)
    }
}

impl ops::SubAssign<f64> for Vector {
    fn sub_assign(&mut self, rhs: f64) {
        sub_assign_num_to_vector(self, rhs)
    }
}

// Multiplication implementations
impl ops::Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        mul_vector_and_scalar(&self, rhs)
    }
}

impl ops::Mul<f64> for &Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        mul_vector_and_scalar(self, rhs)
    }
}

impl ops::MulAssign<f64> for Vector {
    fn mul_assign(&mut self, rhs: f64) {
        mul_assign_num_to_vector(self, rhs)
    }
}

// Division implementations
impl ops::Div<f64> for Vector {
    type Output = Vector;
    fn div(self, rhs: f64) -> Self::Output {
        div_vector_and_scalar(&self, rhs)
    }
}

impl ops::Div<f64> for &Vector {
    type Output = Vector;
    fn div(self, rhs: f64) -> Self::Output {
        div_vector_and_scalar(self, rhs)
    }
}

impl ops::DivAssign<f64> for Vector {
    fn div_assign(&mut self, rhs: f64) {
        div_assign_num_to_vector(self, rhs)
    }
}

// Neg implementations
impl ops::Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Self::Output {
        neg_vector(&self)
    }
}

impl ops::Neg for &Vector {
    type Output = Vector;
    fn neg(self) -> Self::Output {
        neg_vector(self)
    }
}

/// Type alias for [`Vector`]
pub type Point = Vector;

/// Represents a unit vector
// I didn't make this `UtVector(Vector)` because I wanted the fields to be private so that it won't be initializable outside this module
#[derive(Debug, Clone, Copy)]
pub struct UtVector {
    v: Vector,
}

impl Deref for UtVector {
    type Target = Vector;
    fn deref(&self) -> &Self::Target {
        &self.v
    }
}

impl UtVector {
    pub fn relax(self) -> Vector {
        self.v
    }

    pub fn inner(&self) -> &Vector {
        &self.v
    }

    pub fn reflect(&self, normal: &Self) -> Self {
        (self.inner() - normal.inner() * (self.dot(normal) * 2.0)).unit()
    }

    pub fn refract(&self, normal: &Self, refraction_index: f64) -> Self {
        // R  : incident ray
        // R' : transmitted ray
        // n  : normal vector, same side as incident ray (`normal`)
        // n' : normal vector, same side as transmitted ray
        // i  : `refraction_index`

        let incident = self.inner();
        let normal_dir = &-normal.inner();

        let cos_theta = (-incident).dot(normal_dir).min(1.0);

        let r_out_perp = (incident + normal_dir * cos_theta) * refraction_index;
        let r_out_parallel = normal_dir * (1.0 - r_out_perp.len_squared()).abs().sqrt();

        // Return a unit vector
        (r_out_parallel + r_out_perp).unit()
    }
}

// Add implementations
impl ops::Add<UtVector> for Vector {
    type Output = Vector;
    fn add(self, rhs: UtVector) -> Self::Output {
        add_vector_to_utvector(&self, &rhs)
    }
}

impl ops::Add<Vector> for UtVector {
    type Output = Vector;
    fn add(self, rhs: Vector) -> Self::Output {
        add_vector_to_utvector(&rhs, &self)
    }
}

// Neg implementations
impl ops::Neg for UtVector {
    type Output = UtVector;
    fn neg(self) -> Self::Output {
        neg_utvector(&self)
    }
}

impl ops::Neg for &UtVector {
    type Output = UtVector;
    fn neg(self) -> Self::Output {
        neg_utvector(self)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Length of given `Vector` is not 1.0")]
    NotUnitVector,
}
