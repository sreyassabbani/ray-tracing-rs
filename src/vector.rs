//! Module defining [`Vector<T, N>`], and variants [`Point<T, 3>`], ...

use crate::number::Numeric;
use std::{fmt, ops};

#[derive(Clone)]
pub struct Vector<T, const N: usize> {
    pub(super) entries: Box<[T; N]>,
}

impl<T, const N: usize> fmt::Debug for Vector<T, N>
where
    T: Numeric<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.entries.len() - 1 {
            write!(f, "{:?} ", self[i])?;
        }
        write!(f, "{:?}", self[self.entries.len() - 1])?;
        Ok(())
    }
}

impl<T, const N: usize> FromIterator<T> for Vector<T, N>
where
    T: Numeric<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut collection = Self::_new();

        for (i, ele) in iter.into_iter().enumerate() {
            collection[i] = ele;
        }

        collection
    }
}

impl<T, const N: usize> std::ops::Index<usize> for Vector<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<T, const N: usize> std::ops::IndexMut<usize> for Vector<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<T, const N: usize> From<[T; N]> for Vector<T, N> {
    fn from(value: [T; N]) -> Self {
        Self {
            entries: Box::new(value),
        }
    }
}

impl<T, const N: usize> Vector<T, N>
where
    T: Numeric<T>,
{
    /// An internal method for instantiating a [`Vector<T, N>`] type.
    pub(super) fn _new() -> Self {
        Self {
            // `core::array::from_fn` avoids `Copy`
            entries: Box::new(core::array::from_fn(|_| T::default())),
        }
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> T {
        // TODO: look into monoids for `additive_identity`
        self.entries
            .iter()
            .zip(other.entries.iter())
            .map(|(&l, &r)| l * r)
            .fold(T::additive_identity(), |acc, x| acc + x)
    }

    /// Returns the norm squared of a [`Vector<T, N>`]
    #[inline]
    pub fn norm_squared(&self) -> T {
        self.dot(self)
    }

    /// Norm/length of a vector
    #[inline]
    pub fn norm(&self) -> T {
        T::from(self.norm_squared().into().sqrt())
    }

    #[inline]
    pub fn unit(&self) -> Self {
        self / self.norm()
    }
}

impl<T, const N: usize> ops::Add for Vector<T, N>
where
    T: Numeric<T>,
{
    type Output = Vector<T, N>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.entries
            .iter()
            .zip(rhs.entries.iter())
            .map(|(&l, &r)| l + r)
            .collect()
    }
}

impl<T, const N: usize> ops::Add for &Vector<T, N>
where
    T: Numeric<T>,
{
    type Output = Vector<T, N>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.entries
            .iter()
            .zip(rhs.entries.iter())
            .map(|(&l, &r)| l + r)
            .collect()
    }
}

impl<T, const N: usize> ops::AddAssign<T> for Vector<T, N>
where
    T: Numeric<T>,
{
    #[inline]
    fn add_assign(&mut self, rhs: T) {
        self.entries.map(|e| e + rhs);
    }
}

impl<T, const N: usize> ops::Sub for Vector<T, N>
where
    T: Numeric<T>,
{
    type Output = Vector<T, N>;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.entries
            .iter()
            .zip(rhs.entries.iter())
            .map(|(&l, &r)| l - r)
            .collect()
    }
}

impl<T, const N: usize> ops::Sub for &Vector<T, N>
where
    T: Numeric<T>,
{
    type Output = Vector<T, N>;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.entries
            .iter()
            .zip(rhs.entries.iter())
            .map(|(&l, &r)| l - r)
            .collect()
    }
}

impl<T, const N: usize> ops::Sub<Vector<T, N>> for &Vector<T, N>
where
    T: Numeric<T>,
{
    type Output = Vector<T, N>;
    #[inline]
    fn sub(self, rhs: Vector<T, N>) -> Self::Output {
        self.entries
            .iter()
            .zip(rhs.entries.iter())
            .map(|(&l, &r)| l - r)
            .collect()
    }
}

impl<T, const N: usize> ops::SubAssign<T> for Vector<T, N>
where
    T: Numeric<T>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: T) {
        self.entries.map(|e| e + rhs);
    }
}

// TODO: Left multiplication is blocked by orphan rules
impl<T, const N: usize> ops::Mul<T> for Vector<T, N>
where
    T: Numeric<T>,
{
    type Output = Vector<T, N>;
    /// Provides scalar multiplication of [`Vector<T, N>`]
    #[inline]
    fn mul(self, rhs: T) -> Self::Output {
        Vector::from(self.entries.map(|ele| ele * rhs))
    }
}

impl<T, const N: usize> ops::Mul<T> for &Vector<T, N>
where
    T: Numeric<T>,
{
    type Output = Vector<T, N>;
    /// Provides scalar multiplication of [`Vector<T, N>`]
    #[inline]
    fn mul(self, rhs: T) -> Self::Output {
        Vector::from(self.entries.map(|ele| ele * rhs))
    }
}

impl<T, const N: usize> ops::MulAssign<T> for Vector<T, N>
where
    T: Numeric<T>,
{
    #[inline]
    fn mul_assign(&mut self, rhs: T) {
        self.entries.map(|e| e + rhs);
    }
}

impl<T, const N: usize> ops::Div<T> for Vector<T, N>
where
    T: Numeric<T>,
{
    type Output = Vector<T, N>;
    /// Provides scalar division of [`Vector<T, N>`]
    #[inline]
    fn div(self, rhs: T) -> Self::Output {
        Vector::from(self.entries.map(|ele| ele / rhs))
    }
}

impl<T, const N: usize> ops::Div<T> for &Vector<T, N>
where
    T: Numeric<T>,
{
    type Output = Vector<T, N>;
    /// Provides scalar division of [`&Vector<T, N>`]
    #[inline]
    fn div(self, rhs: T) -> Self::Output {
        Vector::from(self.entries.map(|ele| ele / rhs))
    }
}

impl<T, const N: usize> ops::DivAssign<T> for Vector<T, N>
where
    T: Numeric<T>,
{
    #[inline]
    fn div_assign(&mut self, rhs: T) {
        self.entries.map(|e| e / rhs);
    }
}

/// Exported type alias for [`Vector<T, 3>`]
pub type Point<T> = Vector<T, 3>;

impl<T> Point<T>
where
    T: Numeric<T>,
{
    pub fn new(x: T, y: T, z: T) -> Self {
        Self {
            entries: Box::new([x, y, z]),
        }
    }

    pub fn x(&self) -> T {
        self[0]
    }

    pub fn y(&self) -> T {
        self[1]
    }

    pub fn z(&self) -> T {
        self[2]
    }

    // Pretty sure cross product only exists in 3 (and 7) dimensions
    /// Cross product with another 3D vector ([`Point<T>`])
    pub fn cross(&self, other: &Self) -> Self {
        // Ouch to cache
        // TODO: make this better
        Vector::from([
            self[1] * other[2] - self[2] * other[1],
            self[2] * other[0] - self[0] * other[2],
            self[0] * other[1] - self[1] * other[0],
        ])
    }
}
