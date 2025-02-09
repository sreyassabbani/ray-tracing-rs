use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::ray::Ray;
use crate::vector::{Point, Vector};

use env_logger;
use log::info;

pub fn ppm<T: AsRef<Path>>(path: T) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    // Set up logger
    env_logger::init();

    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio).max(1.0) as i32;

    // Camera
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
    let camera_center = &Point::new(0.0, 0.0, 0.0);

    let viewport_u = &Vector::from([viewport_width, 0.0, 0.0]);
    let viewport_v = &Vector::from([0.0, -viewport_height, 0.0]);

    let pixel_delta_u = &(viewport_u / image_width as f64);
    let pixel_delta_v = &(viewport_v / image_height as f64);

    let viewport_upper_left = camera_center
        - Vector::from([0.0, 0.0, focal_length])
        - viewport_u / 2.0
        - viewport_v / 2.0;
    let pixel00_loc = &(viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5);

    // P3 PPM header
    writeln!(file, "P3")?;
    let image_height = 225;
    let image_width = 400;
    writeln!(file, "{} {}", image_width, image_height)?;
    writeln!(file, "255")?; // The maximum color value for RGB channels in P3

    // Write the pixel data
    for j in 0..image_height {
        info!("Scanlines remaining: {}", image_height - j);
        io::stdout().flush().unwrap();
        for i in 0..image_width {
            let pixel_center =
                pixel00_loc + &(pixel_delta_u * i as f64) + (pixel_delta_v * j as f64);
            let ray_direction = pixel_center.clone() - camera_center.clone();
            let r = Ray::new(&camera_center, &ray_direction);

            let pixel_color = r.color();
            writeln!(file, "{}", pixel_color)?;
        }
    }

    Ok(())
}
