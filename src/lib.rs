//! Re-exports

pub mod color;
pub mod materials;
pub mod objects;
pub mod ray;
pub mod scene;
pub mod vector;

pub use objects::HittableList;
pub use scene::{
    Camera, CameraConfig, CameraPose, ConfigError, ImageOptions, LensSettings,
    PerspectiveProjection,
};
pub use vector::Point;

// Probly should revert back to being private
pub mod utils;
