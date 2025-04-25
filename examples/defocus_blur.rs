//! Example use of the ray tracing library
//!
//! Render a example image of defocus blur

use ray_tracing_rs::color::Color;
use ray_tracing_rs::materials::{Dielectric as Glass, Lambertian as Matte, Metal};
use ray_tracing_rs::objects::Sphere;
use ray_tracing_rs::vector::Vector;
use ray_tracing_rs::{Camera, HittableList, ImageOptions, Point};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up materials
    let material_ground = Matte::new(Color::new(0.8, 0.8, 0.0));
    let material_center = Matte::new(Color::new(0.1, 0.2, 0.5));
    let material_bubble = Glass::new(1.0 / 1.50);
    let material_left = Glass::new(1.50);
    let material_right = Metal::new(Color::new(0.8, 0.6, 0.2), 1.0);

    // Set up objects
    let ground = Sphere::new(Point::new(0.0, -100.5, -1.0), 100.0, material_ground);
    let center = Sphere::new(Point::new(0.0, 0.0, -1.2), 0.5, material_center);
    let left = Sphere::new(Point::new(-1.0, 0.0, -1.0), 0.5, material_left);
    let bubble = Sphere::new(Point::new(-1.0, 0.0, -1.0), 0.4, material_bubble);
    let right = Sphere::new(Point::new(1.0, 0.0, -1.0), 0.5, material_right);

    // Set up the world
    let mut world = HittableList::new();

    world
        .add(ground)?
        .add(center)?
        .add(left)?
        .add(right)?
        .add(bubble)?;

    // Output image config, aspect ratio 16:9
    let image = ImageOptions::new(400, 225).antialias(100);

    // Camera
    let vfov = 20.0;
    let look_from = Point::new(-2.0, 2.0, 1.0);
    let look_at = Point::new(0.0, 0.0, -1.0);
    let up = Vector::new(0.0, 1.0, 0.0);
    let defocus_angle = 10.0;

    let camera = Camera::new(vfov, defocus_angle, look_from, look_at, up, image, world)?;

    dbg!(&camera);

    camera.render("output.ppm")?;

    Ok(())
}
