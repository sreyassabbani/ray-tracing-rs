//! Example use of the ray tracing library.
//!
//! Render the final Ray Tracing in One Weekend scene

use ray_tracing_rs::color::Color;
use ray_tracing_rs::materials::{Dielectric as Glass, Lambertian as Matte, Metal};
use ray_tracing_rs::objects::Sphere;
use ray_tracing_rs::utils::rand::{random, random_range};
use ray_tracing_rs::vector::Vector;
use ray_tracing_rs::{Camera, HittableList, ImageOptions, Point};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut world = HittableList::new();
    let ground_material = Matte::new(Color::new(0.5, 0.5, 0.5));
    world.add(Sphere::new(
        Point::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    ))?;

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random();
            let center = Point::new(a as f64 + 0.9 * random(), 0.2, b as f64 + 0.9 * random());

            if (center - Point::new(4.0, 0.2, 0.0)).len() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Color::random() * Color::random();
                    let mat = Matte::new(albedo);
                    world.add(Sphere::new(center, 0.2, mat))?;
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = random_range(0.0, 0.5);
                    let mat = Metal::new(albedo, fuzz);
                    world.add(Sphere::new(center, 0.2, mat))?;
                } else {
                    // glass
                    let mat = Glass::new(1.5);
                    world.add(Sphere::new(center, 0.2, mat))?;
                }
            }
        }
    }

    let material1 = Glass::new(1.5);
    world.add(Sphere::new(Point::new(0.0, 1.0, 0.0), 1.0, material1))?;

    let material2 = Matte::new(Color::new(0.4, 0.2, 0.1));
    world.add(Sphere::new(Point::new(-4.0, 1.0, 0.0), 1.0, material2))?;

    let material3 = Metal::new(Color::new(0.7, 0.6, 0.5), 0.0);
    world.add(Sphere::new(Point::new(4.0, 1.0, 0.0), 1.0, material3))?;

    // Output image config, aspect ratio 16:9
    let image = ImageOptions::new(1200, 675).antialias(50);

    // Camera
    let vfov = 20.0;
    let look_from = Point::new(13.0, 2.0, 3.0);
    let look_at = Point::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 1.0, 0.0);
    let defocus_angle = 0.6;
    let focus_dist = 10.0;

    let camera = Camera::new(
        vfov,
        defocus_angle,
        focus_dist,
        look_from,
        look_at,
        up,
        image,
        world,
    )?;

    camera.render("output.ppm")?;

    Ok(())
}
