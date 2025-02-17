//! Module exposing the API for [`Camera`], [`ImageOptions`], [`ViewportOptions`]
//! Contains logic for writing in PPM format

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use env_logger;
use log::info;

use thiserror::Error;

use rayon::prelude::*;

use crate::color::Color;
use crate::objects::HittableList;
use crate::ray::Ray;
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

    /// Use for smoothening rough edges and color differences.
    ///
    /// Specify samples per pixel (SPP). Specifying 0 will result in [`AntialiasOptions::Disabled`], which is the default for [`ImageOptions`]
    ///
    /// * Antialiasing is off by default
    ///
    /// ```rs
    /// // Enable antialiasing with 10 samples per pixel
    /// let image = ImageOptions::new(width, height).antialias(10);
    /// ```
    pub fn antialias(mut self, spp: u32) -> Self {
        if spp == 0 {
            self.antialias = AntialiasOptions::Disabled;
        } else {
            self.antialias = AntialiasOptions::Enabled(spp);
        }
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
    u: Vector,
    v: Vector,
    pixel_delta_u: Vector,
    pixel_delta_v: Vector,
    viewport_upper_left: Point,
    pixel00_loc: Point,
    pixel_samples_scale: Option<f64>,
}

impl ComputedData {
    fn new(
        viewport_options: &ViewportOptions,
        image_options: &ImageOptions,
        center: &Point,
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
pub struct RenderOptions {
    parallel: ParallelOptions,
}

#[derive(Clone, Debug)]
pub enum ParallelOptions {
    /// Render all of the grid in parallel. Write everything to the file afterwards, sequentially.
    AllAtOnce,
    /// Render the first row, write to the file. Render the second row, write to the file, and so on.
    ByRows,
    /// Render in series (sequentially). Once pixel at a time and write immediately after being computed.
    Series,
}

impl RenderOptions {
    pub fn new() -> Self {
        Self {
            parallel: ParallelOptions::AllAtOnce,
        }
    }

    pub fn parallel(mut self, config: ParallelOptions) -> Self {
        self.parallel = config;
        self
    }
}

#[derive(Clone)]
pub struct Camera {
    center: Point,
    focal_length: f64,
    viewport_options: ViewportOptions,
    image_options: ImageOptions,
    render_options: RenderOptions,
    computed_data: ComputedData,
    world: HittableList,
}

impl Camera {
    /// Can only call once
    pub fn new(
        center: Point,
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

        // Set up logger only once
        env_logger::init();

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

    pub fn update_image_options(&mut self, image_options: ImageOptions) {
        self.image_options = image_options;

        // Update computed data -- REALLY BAD -- TODO refactor
        self.computed_data.pixel_samples_scale = match image_options.antialias {
            AntialiasOptions::Disabled => None,
            AntialiasOptions::Enabled(samples_per_pixel) => Some(1.0 / samples_per_pixel as f64),
        };
    }

    // Need to not make public
    pub fn update_render_options(&mut self, render_options: RenderOptions) {
        self.render_options = render_options;

        // Look at `Self::update_image_options`, implement logic like that as necessary
    }

    pub fn render<T: AsRef<Path>>(&self, path: T) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        self.write_ppm_p3_header(&mut file)?;

        use ParallelOptions::*;
        match self.render_options.parallel {
            AllAtOnce => self.render_parallel_all(&mut file)?,
            ByRows => self.render_parallel_by_rows(&mut file)?,
            Series => self.render_sequential(&mut file)?,
        };

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

    /// Internal inlined function that is called when `render_options`: [`RenderOptions`] of [`Camera`] has the `parallel` field set to [`ParallelOptions::AllAtOnce`]
    #[inline]
    fn render_parallel_all(&self, file: &mut fs::File) -> Result<(), Box<dyn std::error::Error>> {
        let mut pixels = vec![
            Color::new(0.0, 0.0, 0.0);
            (self.image_options.height * self.image_options.width) as usize
        ];

        pixels.par_iter_mut().enumerate().for_each(|(i, v)| {
            let x = (i as u32) % self.image_options.width;
            let y = (i as u32) / self.image_options.width;
            *v = self.pixel_color_at(x, y);
        });

        info!("Finished calculations!");

        // Write the pixel data
        for i in 0..(pixels.len() as u32) {
            if i % self.image_options.width == 0 {
                info!(
                    "Scanlines remaining to write: {}",
                    self.image_options.height - (i / self.image_options.width)
                );
            }
            writeln!(file, "{}", pixels[i as usize])?;
        }
        Ok(())
    }

    /// Internal inlined function that is called when `render_options`: [`RenderOptions`] of [`Camera`] has the `parallel` field set to [`ParallelOptions::ByRows`]
    #[inline]
    fn render_parallel_by_rows(
        &self,
        file: &mut fs::File,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for j in 0..self.image_options.height {
            info!("Scanlines remaining: {}", self.image_options.height - j);
            io::stdout().flush().unwrap();

            // Calculate pixel data
            let row_pixels: Vec<_> = (0..self.image_options.width)
                .into_par_iter()
                .map(|i| self.pixel_color_at(i, j))
                .collect();

            // Write the pixel data
            for pixel_color in row_pixels {
                writeln!(file, "{}", pixel_color)?;
            }
        }
        Ok(())
    }

    /// Internal inlined function that is called when `render_options`: [`RenderOptions`] of [`Camera`] has the `parallel` field set to [`ParallelOptions::Series`]
    #[inline]
    fn render_sequential(&self, file: &mut fs::File) -> Result<(), Box<dyn std::error::Error>> {
        // Write the pixel data
        for j in 0..self.image_options.height {
            info!(
                "Scanlines remaining: {}",
                self.image_options.height - (j as u32 / self.image_options.width)
            );
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
                let r = Ray::new(&self.center, ray_direction.unit());
                pixel_color += r.color(&self.world, 50);
            }
            Enabled(samples_per_pixel) => {
                for _ in 0..samples_per_pixel {
                    let r = self.get_antialiasing_ray(i, j);
                    // Should never panic
                    pixel_color +=
                        r.color(&self.world, 50) * self.computed_data.pixel_samples_scale.unwrap();
                }
            }
        }
        pixel_color
    }

    #[inline]
    fn get_pixel_center_coordinates(&self, i: u32, j: u32) -> Point {
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
        Ray::new(&self.center, point_to.unit())
    }

    /// Internal method for generating a random vector inside of a unit square
    #[inline]
    fn sample_square() -> Vector {
        Vector::new(
            rand::random_range(-0.5, 0.5),
            rand::random_range(-0.5, 0.5),
            0.0,
        )
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Image and viewport aspect ratios are not equal")]
    MismatchedImageViewportAspectRatios,
}
