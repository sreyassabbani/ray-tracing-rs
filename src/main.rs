use ray_tracing_rs::color::Color;
use ray_tracing_rs::material::{Lambertian, Metal};
use ray_tracing_rs::objects::Sphere;
use ray_tracing_rs::{Camera, HittableList, ImageOptions, Point, ViewportOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up materials
    let material_ground = Lambertian::new(Color::new(0.8, 0.8, 0.0));
    let material_center = Lambertian::new(Color::new(0.1, 0.2, 0.5));
    let material_left = Metal::new(Color::new(0.8, 0.8, 0.8), 0.3);
    let material_right = Metal::new(Color::new(0.8, 0.6, 0.2), 1.0);

    let ground = Sphere::new(Point::new(0.0, -100.5, -1.0), 100.0, material_ground);
    let center = Sphere::new(Point::new(0.0, 0.0, -1.2), 0.5, material_center);
    let left = Sphere::new(Point::new(-1.0, 0.0, -1.0), 0.5, material_left);
    let right = Sphere::new(Point::new(1.0, 0.0, -1.0), 0.5, material_right);

    let mut world = HittableList::new();
    world.add(ground)?;
    world.add(center)?;
    world.add(left)?;
    world.add(right)?;

    // Output image config, aspect ratio 16:9
    let image = ImageOptions::new(400, 225).antialias(100);

    // Viewport config
    let viewport = ViewportOptions::new(image.aspect_ratio() * 2.0, 2.0);

    // Camera
    let camera = Camera::new(Point::new(0.0, 0.0, 0.0), 1.0, viewport, image, world)?;

    camera.render("output.ppm")?;

    Ok(())
}
