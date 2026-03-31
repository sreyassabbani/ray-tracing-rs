//! Re-exports

pub mod color;
pub mod materials;
pub mod objects;
pub mod ray;
pub mod scene;
pub mod vector;

pub use objects::HittableList;
pub use scene::{
    Camera, CameraConfig, CameraModel, CameraPose, ConfigError, ImageOptions, PerspectiveProjection,
};
pub use vector::Point;

mod utils;
