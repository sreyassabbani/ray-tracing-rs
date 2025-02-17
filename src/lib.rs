//! Re-exports

pub mod color;
pub mod materials;
pub mod objects;
pub mod ray;
pub mod scene;
pub mod vector;

pub use objects::HittableList;
pub use scene::{Camera, ImageOptions, ViewportOptions};
pub use vector::Point;

mod utils;
