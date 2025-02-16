//! Re-exports

pub mod color;
pub mod material;
pub mod number;
pub mod objects;
pub mod ray;
pub mod scene;
pub mod vector;

pub use ray::HittableList;
pub use scene::{Camera, ImageOptions, ViewportOptions};
pub use vector::Point;

mod utils;
