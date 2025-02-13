use ray_tracing_rs::objects::Sphere;
use ray_tracing_rs::{Camera, HittableList, ImageOptions, Point, ViewportOptions};

use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the world
    let mut world = HittableList::new();
    let sphere = Sphere::new(Point::new(0.0, 0.0, -1.0), 0.5);
    let ground = Sphere::new(Point::new(0.0, -100.5, -1.0), 100.0);
    world.add(Arc::new(sphere))?;
    world.add(Arc::new(ground))?;

    // Output image config, aspect ratio 16:9
    let image = ImageOptions::new(400, 225).enable_antialias(100);

    // Viewport config
    let viewport = ViewportOptions::new(image.aspect_ratio() * 2.0, 2.0);

    // Camera
    let camera = Camera::new(Point::new(0.0, 0.0, 0.0), 1.0, viewport, image, world)?;

    camera.render("output.ppm")?;

    Ok(())
}
