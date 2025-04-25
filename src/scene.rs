//! Module exposing the API for [`Camera`], [`ImageOptions`], [`ViewportOptions`]
//! Contains logic for writing in PPM format

use std::{
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

use env_logger;
use log::info;

use thiserror::Error;

use rayon::prelude::*;

use crate::color::Color;
use crate::objects::HittableList;
use crate::ray::Ray;
use crate::utils::{self, rand};
use crate::vector::{Point, UtVector, Vector};

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

#[derive(Clone, Debug)]
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
            parallel: ParallelOptions::ByRows,
        }
    }

    pub fn parallel(mut self, config: ParallelOptions) -> Self {
        self.parallel = config;
        self
    }
}
/// Lightweight wrapper around `T`because `T` be `Option<T>`
#[derive(Clone, Default)]
pub enum BuildField<T> {
    Init(T),
    #[default]
    Uninit,
}

#[derive(Clone, Default)]
#[allow(unused)]
pub struct CameraBuilder {
    center: BuildField<Point>,
    focal_length: BuildField<f64>,
    vfov: BuildField<f64>,
    v: BuildField<UtVector>,
    u: BuildField<UtVector>,
    w: BuildField<UtVector>,
    viewport_u: BuildField<Vector>,
    viewport_v: BuildField<Vector>,
    pixel_delta_u: BuildField<Vector>,
    pixel_delta_v: BuildField<Vector>,
    defocus_angle: BuildField<f64>,
    viewport_upper_left: BuildField<Point>,
    pixel00_loc: BuildField<Point>,
    pixel_samples_scale: BuildField<Option<f64>>,
    image_options: BuildField<ImageOptions>,
    render_options: BuildField<RenderOptions>,
    world: BuildField<HittableList>,
}

impl CameraBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug)]
struct Viewport {
    u: Vector,
    v: Vector,
    pixel_delta_u: Vector,
    pixel_delta_v: Vector,
    upper_left: Point,
}

#[derive(Clone)]
pub struct Camera {
    center: Point,
    focal_vector: Vector,
    viewport: Viewport,
    defocus_angle: f64,
    image_options: ImageOptions,
    render_options: RenderOptions,
    world: HittableList,
}

impl fmt::Debug for Camera {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Camera")
            .field("center", &self.center)
            .field("focal_vector", &self.focal_vector)
            .field("viewport", &self.viewport)
            .field("defocus_angle", &self.defocus_angle)
            .field("image_options", &self.image_options)
            .field("render_options", &self.render_options)
            // intentionally skipping `.world`
            .finish()
    }
}

impl Camera {
    /// Can only call once
    pub fn new(
        vfov: f64,
        defocus_angle: f64,
        look_from: Point,
        look_at: Point,
        up: Vector,
        image_options: ImageOptions,
        world: HittableList,
    ) -> Result<Self, Error> {
        let render_options = RenderOptions::new();

        // Set up logger only once
        env_logger::init();

        let theta = (vfov / 180.0) * std::f64::consts::PI;

        let h = (theta / 2.0).tan();
        let focal_vector = look_from - look_at;
        let focal_length = focal_vector.len();
        let viewport_height = 2.0 * h * focal_length;
        let viewport_width =
            viewport_height * (image_options.width as f64 / image_options.height as f64);

        let w = focal_vector.unit();
        let u = up.cross(&w).unit() * viewport_width;
        let v = u.cross(&w).assert_unit_unsafe() * viewport_height / viewport_width;

        let pixel_delta_u = u / image_options.width as f64;
        let pixel_delta_v = v / image_options.height as f64;

        let viewport_upper_left = look_from - focal_vector - (u + v) / 2.0;

        Ok(Self {
            center: look_from,
            focal_vector,
            viewport: Viewport {
                u,
                v,
                pixel_delta_u,
                pixel_delta_v,
                upper_left: viewport_upper_left,
            },
            defocus_angle,
            image_options,
            render_options,
            world,
        })
    }

    pub fn defocus_radius(&self) -> f64 {
        self.focal_vector.len() * (utils::degrees_to_radians(self.defocus_angle / 2.0)).tan()
    }

    pub fn defocus_disk_u(&self) -> Vector {
        self.viewport.u * self.defocus_radius()
    }

    pub fn defocus_disk_v(&self) -> Vector {
        self.viewport.v * self.defocus_radius()
    }

    pub fn update_image_options(&mut self, image_options: ImageOptions) {
        self.image_options = image_options;
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

    fn pixel_color_at(&self, i: u32, j: u32) -> Color {
        let mut pixel_color = Color::new(0.0, 0.0, 0.0);

        use AntialiasOptions::*;
        match self.image_options.antialias {
            Disabled => {
                let pixel_center = self.get_pixel_center_coordinates(i, j);
                let ray_direction = pixel_center - self.center;
                let r = Ray::new(&self.center, ray_direction.unit());
                pixel_color += r.color(&self.world, 50);
            }
            Enabled(samples_per_pixel) => {
                for _ in 0..samples_per_pixel {
                    let (ray_origin, ray_dir) = self.get_antialiasing_ray_components(i, j);
                    let r = Ray::new(&ray_origin, ray_dir);
                    // Should never panic
                    pixel_color += r.color(&self.world, 50) * (1.0 / samples_per_pixel as f64);
                }
            }
        }
        pixel_color
    }

    fn get_pixel_center_coordinates(&self, i: u32, j: u32) -> Point {
        self.viewport.upper_left
            + (self.viewport.pixel_delta_u * (i as f64 + 0.5))
            + (self.viewport.pixel_delta_v * (j as f64 + 0.5))
    }

    /// Gives a [`Ray`] that is nearby the neighborhood of `i` and `j`. Specifically, at most 0.5 away from real location
    fn get_antialiasing_ray_components(&self, i: u32, j: u32) -> (Point, UtVector) {
        let offset = Self::sample_square();
        // let point_to = self.get_pixel_center_coordinates(i, j) - offset;
        let point_to = &self.viewport.upper_left
            + (self.viewport.pixel_delta_u * (i as f64 + offset.x() + 0.5))
            + (self.viewport.pixel_delta_v * (j as f64 + offset.y() + 0.5));
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center.clone()
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = (point_to - ray_origin).unit();
        (ray_origin, ray_direction)
    }

    /// Internal method for generating a random vector inside of a unit square
    fn sample_square() -> Vector {
        Vector::new(
            rand::random_range(-0.5, 0.5),
            rand::random_range(-0.5, 0.5),
            0.0,
        )
    }

    fn defocus_disk_sample(&self) -> Point {
        // Returns a random point in the camera defocus disk.
        let p = Vector::random_unit();
        self.center + (self.defocus_disk_u() * p.x()) + (self.defocus_disk_v() * p.y())
    }
}

#[derive(Error, Debug)]
pub enum Error {}
