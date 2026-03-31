use ray_tracing_rs::color::Color;
use ray_tracing_rs::materials::Lambertian;
use ray_tracing_rs::objects::Sphere;
use ray_tracing_rs::vector::Vector;
use ray_tracing_rs::{
    Camera, CameraConfig, CameraModel, CameraPose, ConfigError, HittableList, ImageOptions,
    PerspectiveProjection, Point,
};

fn test_camera(look_from: Point, look_at: Point) -> Camera {
    let pose = CameraPose::look_at(look_from, look_at, Vector::new(0.0, 1.0, 0.0)).unwrap();
    let image = ImageOptions::new(8, 4).unwrap();
    let projection = PerspectiveProjection::new(60.0).unwrap();
    let model = CameraModel::pinhole(1.0).unwrap();
    Camera::new(CameraConfig::new(pose, image, projection, model))
}

fn blank_world() -> HittableList {
    HittableList::new()
}

fn sphere_world() -> HittableList {
    let mut world = HittableList::new();
    let material = Lambertian::new(Color::new(0.8, 0.3, 0.3));
    world
        .add(Sphere::new(Point::new(0.0, 0.0, -1.0), 0.5, material))
        .unwrap();
    world
}

fn pixels_to_strings(camera: &Camera, world: &HittableList) -> Vec<String> {
    camera
        .render_in_memory(world)
        .into_iter()
        .map(|pixel| pixel.to_string())
        .collect()
}

#[test]
fn image_options_reject_zero_dimensions() {
    assert_eq!(
        ImageOptions::new(0, 1).unwrap_err(),
        ConfigError::InvalidImageDimensions
    );
    assert_eq!(
        ImageOptions::new(1, 0).unwrap_err(),
        ConfigError::InvalidImageDimensions
    );
}

#[test]
fn perspective_projection_rejects_invalid_values() {
    assert_eq!(
        PerspectiveProjection::new(0.0).unwrap_err(),
        ConfigError::InvalidFieldOfView
    );
    assert_eq!(
        PerspectiveProjection::new(180.0).unwrap_err(),
        ConfigError::InvalidFieldOfView
    );
    assert_eq!(
        PerspectiveProjection::new(f64::NAN).unwrap_err(),
        ConfigError::InvalidFieldOfView
    );
}

#[test]
fn camera_model_rejects_invalid_values() {
    assert_eq!(
        CameraModel::pinhole(0.0).unwrap_err(),
        ConfigError::InvalidViewportDistance
    );
    assert_eq!(
        CameraModel::thin_lens(1.0, -0.1).unwrap_err(),
        ConfigError::InvalidDefocusAngle
    );
    assert_eq!(
        CameraModel::thin_lens(1.0, 0.0).unwrap_err(),
        ConfigError::InvalidDefocusAngle
    );
    assert_eq!(
        CameraModel::thin_lens(f64::NAN, 0.5).unwrap_err(),
        ConfigError::InvalidFocusDistance
    );
}

#[test]
fn camera_pose_rejects_degenerate_view_direction() {
    assert_eq!(
        CameraPose::look_at(
            Point::new(0.0, 0.0, 0.0),
            Point::new(0.0, 0.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
        )
        .unwrap_err(),
        ConfigError::DegenerateViewDirection
    );
}

#[test]
fn camera_pose_rejects_parallel_up_vector() {
    assert_eq!(
        CameraPose::look_at(
            Point::new(0.0, 0.0, 0.0),
            Point::new(0.0, 0.0, -1.0),
            Vector::new(0.0, 0.0, 1.0),
        )
        .unwrap_err(),
        ConfigError::UpVectorParallelToView
    );
}

#[test]
fn one_camera_can_render_multiple_worlds() {
    let camera = test_camera(Point::new(0.0, 0.0, 0.0), Point::new(0.0, 0.0, -1.0));
    let blank = blank_world();
    let sphere = sphere_world();

    let blank_pixels = pixels_to_strings(&camera, &blank);
    let sphere_pixels = pixels_to_strings(&camera, &sphere);

    assert_eq!(blank_pixels.len(), 32);
    assert_eq!(sphere_pixels.len(), 32);
    assert_ne!(blank_pixels, sphere_pixels);
}

#[test]
fn multiple_cameras_can_render_the_same_world() {
    let world = sphere_world();
    let camera_a = test_camera(Point::new(0.0, 0.0, 0.0), Point::new(0.0, 0.0, -1.0));
    let camera_b = test_camera(Point::new(0.5, 0.2, 0.0), Point::new(0.0, 0.0, -1.0));

    let pixels_a = pixels_to_strings(&camera_a, &world);
    let pixels_b = pixels_to_strings(&camera_b, &world);

    assert_eq!(pixels_a.len(), 32);
    assert_eq!(pixels_b.len(), 32);
    assert_ne!(pixels_a, pixels_b);
}
