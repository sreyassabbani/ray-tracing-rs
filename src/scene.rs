//! Module exposing the API for [`Camera`], [`ImageOptions`], [`ViewportOptions`]
//! Contains logic for writing in PPM format

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use env_logger;
use log::info;

use thiserror::Error;

use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::color::Color;
use crate::ray::{HittableList, Ray};
use crate::utils::rand;
use crate::vector::{Point, Vector};

/// [`ImageOptions`] can be used to configure a [`Camera`].
///
/// * when initializing, the image aspect ratio needs to be the same as the viewport aspect ratio or `Camera::new` will fail.
#[derive(Copy, Clone, Debug)]
pub struct ImageOptions {
    width: u32,
    height: u32,
    antialias: AntialiasOptions,
}

/// Can be used as additional configuration for [`ImageOptions`]
///
/// ```rs
/// // Enable antialiasing with 10 samples per pixel
/// let image = ImageOptions::new(width, height).enable_antialias(10);
/// ```
#[derive(Debug, Clone, Copy)]
enum AntialiasOptions {
    Disabled,
    Enabled(u32),
}

impl ImageOptions {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            antialias: AntialiasOptions::Disabled,
        }
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    /// Use for smoothening rough edges and color differences; need to specify samples per pixel
    ///
    /// ```rs
    /// // Enable antialiasing with 10 samples per pixel
    /// let image = ImageOptions::new(width, height).enable_antialias(10);
    /// ```
    pub fn enable_antialias(mut self, samples_per_pixel: u32) -> Self {
        self.antialias = AntialiasOptions::Enabled(samples_per_pixel);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportOptions {
    width: f64,
    height: f64,
}

impl ViewportOptions {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width / self.height
    }
}

/// Internal struct for handling information about the viewport
/// * Not to be confused with [`ViewportOptions`], which is what the user may configure. The values of everything in this struct is completely determined by [`ViewportOptions`]
#[derive(Clone)]
struct ComputedData {
    u: Vector<f64, 3>,
    v: Vector<f64, 3>,
    pixel_delta_u: Vector<f64, 3>,
    pixel_delta_v: Vector<f64, 3>,
    viewport_upper_left: Point<f64>,
    pixel00_loc: Point<f64>,
    pixel_samples_scale: Option<f64>,
}

impl ComputedData {
    fn new(
        viewport_options: &ViewportOptions,
        image_options: &ImageOptions,
        center: &Point<f64>,
        focal_length: f64,
    ) -> Self {
        let u = Vector::from([viewport_options.width, 0.0, 0.0]);
        let v = Vector::from([0.0, -viewport_options.height, 0.0]);

        let pixel_delta_u = &u / image_options.width as f64;
        let pixel_delta_v = &v / image_options.height as f64;

        let viewport_upper_left =
            center - &Vector::from([0.0, 0.0, focal_length]) - &u / 2.0 - &v / 2.0;
        let pixel00_loc = &viewport_upper_left + &((&pixel_delta_u + &pixel_delta_v) * 0.5);

        let pixel_samples_scale = match image_options.antialias {
            AntialiasOptions::Disabled => None,
            AntialiasOptions::Enabled(samples_per_pixel) => Some(1.0 / samples_per_pixel as f64),
        };

        Self {
            u,
            v,
            pixel_delta_u,
            pixel_delta_v,
            viewport_upper_left,
            pixel00_loc,
            pixel_samples_scale,
        }
    }
}

#[derive(Clone)]
pub(crate) struct RenderOptions {
    parallel: bool,
}

impl RenderOptions {
    pub fn new() -> Self {
        Self { parallel: true }
    }

    pub(crate) fn parallel(&mut self, is_parallel: bool) {
        self.parallel = is_parallel;
    }
}

#[derive(Clone)]
pub struct Camera {
    center: Point<f64>,
    focal_length: f64,
    viewport_options: ViewportOptions,
    image_options: ImageOptions,
    render_options: RenderOptions,
    computed_data: ComputedData,
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

        let computed_data =
            ComputedData::new(&viewport_options, &image_options, &center, focal_length);

        let render_options = RenderOptions::new();

        Ok(Self {
            center,
            focal_length,
            viewport_options,
            image_options,
            render_options,
            computed_data,
            world,
        })
    }

    pub(crate) fn set_render_options(&mut self, render_options: RenderOptions) {
        self.render_options = render_options;
    }

    pub fn render<T: AsRef<Path>>(&self, path: T) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        // Set up logger
        env_logger::init();

        self.write_ppm_p3_header(&mut file)?;

        if self.render_options.parallel {
            self.parallelized_render(&mut file)?;
        } else {
            self.sequential_render(&mut file)?;
        }

        Ok(())
    }

    /// Internal function to write P3 PPM header
    #[inline]
    fn write_ppm_p3_header(&self, file: &mut fs::File) -> Result<(), Box<dyn std::error::Error>> {
        // P3 PPM header
        writeln!(file, "P3")?;
        writeln!(
            file,
            "{} {}",
            self.image_options.width, self.image_options.height
        )?;
        writeln!(file, "255")?; // The maximum color value for RGB channels in P3

        Ok(())
    }

    /// Internal function to handle rendering with [`Rayon`]
    #[inline]
    fn parallelized_render(&self, file: &mut fs::File) -> Result<(), Box<dyn std::error::Error>> {
        // Write the pixel data
        for j in 0..self.image_options.height {
            info!("Scanlines remaining: {}", self.image_options.height - j);
            io::stdout().flush().unwrap();
            let row_pixels: Vec<_> = (0..self.image_options.width)
                .into_par_iter()
                .map(|i| self.pixel_color_at(i, j))
                .collect();
            for pixel_color in row_pixels {
                writeln!(file, "{}", pixel_color)?;
            }
        }
        Ok(())
    }

    /// Internal function to handle sequential rendering. Mainly used for benchmarking parallelized render
    #[inline]
    fn sequential_render(&self, file: &mut fs::File) -> Result<(), Box<dyn std::error::Error>> {
        // Write the pixel data
        for j in 0..self.image_options.height {
            info!("Scanlines remaining: {}", self.image_options.height - j);
            io::stdout().flush().unwrap();
            for i in 0..self.image_options.width {
                let pixel_color = self.pixel_color_at(i, j);
                writeln!(file, "{}", pixel_color)?;
            }
        }

        Ok(())
    }

    #[inline]
    fn pixel_color_at(&self, i: u32, j: u32) -> Color {
        let mut pixel_color = Color::new(0.0, 0.0, 0.0);

        use AntialiasOptions::*;
        match self.image_options.antialias {
            Disabled => {
                let pixel_center = self.get_pixel_center_coordinates(i, j);
                let ray_direction = &pixel_center - &self.center;
                let r = Ray::new(&self.center, ray_direction);
                pixel_color += r.color(&self.world);
            }
            Enabled(samples_per_pixel) => {
                for _ in 0..samples_per_pixel {
                    let r = self.get_antialiasing_ray(i, j);
                    pixel_color +=
                        r.color(&self.world) * self.computed_data.pixel_samples_scale.unwrap();
                }
            }
        }
        pixel_color
    }

    #[inline]
    fn get_pixel_center_coordinates(&self, i: u32, j: u32) -> Point<f64> {
        &self.computed_data.pixel00_loc
            + &(&self.computed_data.pixel_delta_u * i as f64)
            + (&self.computed_data.pixel_delta_v * j as f64)
    }

    /// Gives a [`Ray`] that is nearby the neighborhood of `i` and `j`. Specifically, at most 0.5 away from real location
    #[inline]
    fn get_antialiasing_ray(&self, i: u32, j: u32) -> Ray {
        let offset = Self::sample_square();
        // let point_to = self.get_pixel_center_coordinates(i, j) - offset;
        let point_to = &self.computed_data.pixel00_loc
            + &(&self.computed_data.pixel_delta_u * (i as f64 + offset.x()))
            + (&self.computed_data.pixel_delta_v * (j as f64 + offset.y()));
        Ray::new(&self.center, point_to)
    }

    /// Internal method for generating a random vector inside of a unit square
    #[inline]
    fn sample_square() -> Vector<f64, 3> {
        Vector::new(rand::random(-0.5, 0.5), rand::random(-0.5, 0.5), 0.0)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Image and viewport aspect ratios are not equal")]
    MismatchedImageViewportAspectRatios,
}
