pub mod color;
pub mod number;
pub mod objects;
pub mod ray;
pub mod scene;
pub mod vector;

// Re-exports
pub use ray::HittableList;
pub use scene::{Camera, ImageOptions, ViewportOptions};
pub use vector::Point;

mod utils;
