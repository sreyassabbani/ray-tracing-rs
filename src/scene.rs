use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use env_logger;
use log::info;
use thiserror::Error;

use crate::ray::{HittableList, Ray};
use crate::vector::{Point, Vector};

#[derive(Copy, Clone, Debug)]
pub struct ImageOptions {
    pub(super) width: u32,
    pub(super) height: u32,
}

impl ImageOptions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportOptions {
    pub(super) width: f64,
    pub(super) height: f64,
}

impl ViewportOptions {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width / self.height
    }
}

#[derive(Clone)]
pub struct Camera {
    center: Point<f64>,
    focal_length: f64,
    viewport_options: ViewportOptions,
    image_options: ImageOptions,
    world: HittableList,
}

impl Camera {
    pub fn new(
        center: Point<f64>,
        focal_length: f64,
        viewport_options: ViewportOptions,
        image_options: ImageOptions,
        world: HittableList,
    ) -> Result<Self, Error> {
        if image_options.aspect_ratio() != viewport_options.aspect_ratio() {
            return Err(Error::MismatchedImageViewportAspectRatios);
        }
        Ok(Self {
            center,
            focal_length,
            viewport_options,
            image_options,
            world,
        })
    }

    pub fn render<T: AsRef<Path>>(&self, path: T) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        // Set up logger
        env_logger::init();

        let viewport_u = &Vector::from([self.viewport_options.width, 0.0, 0.0]);
        let viewport_v = &Vector::from([0.0, -self.viewport_options.height, 0.0]);

        let pixel_delta_u = &(viewport_u / self.image_options.width as f64);
        let pixel_delta_v = &(viewport_v / self.image_options.height as f64);

        let viewport_upper_left = &self.center
            - &Vector::from([0.0, 0.0, self.focal_length])
            - viewport_u / 2.0
            - viewport_v / 2.0;
        let pixel00_loc = &(viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5);

        // P3 PPM header
        writeln!(file, "P3")?;
        writeln!(
            file,
            "{} {}",
            self.image_options.width, self.image_options.height
        )?;
        writeln!(file, "255")?; // The maximum color value for RGB channels in P3

        // Write the pixel data
        for j in 0..self.image_options.height {
            info!("Scanlines remaining: {}", self.image_options.height - j);
            io::stdout().flush().unwrap();
            for i in 0..self.image_options.width {
                let pixel_center =
                    pixel00_loc + &(pixel_delta_u * i as f64) + (pixel_delta_v * j as f64);
                let ray_direction = pixel_center.clone() - self.center.clone();
                let r = Ray::new(&self.center, &ray_direction);

                let pixel_color = r.color(&self.world);
                writeln!(file, "{}", pixel_color)?;
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Image and viewport aspect ratios are not equal")]
    MismatchedImageViewportAspectRatios,
}
