//! Camera configuration and rendering API.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

use log::info;
use rayon::prelude::*;
use thiserror::Error;

use crate::color::Color;
use crate::objects::HittableList;
use crate::ray::Ray;
use crate::utils::{self, rand};
use crate::vector::{Point, UtVector, Vector};

/// [`ImageOptions`] configures a rendered image.
#[derive(Copy, Clone, Debug)]
pub struct ImageOptions {
    width: u32,
    height: u32,
    antialias: AntialiasOptions,
}

/// Can be used as additional configuration for [`ImageOptions`].
#[derive(Debug, Clone, Copy)]
enum AntialiasOptions {
    Disabled,
    Enabled(u32),
}

impl ImageOptions {
    /// Create a new set of image options.
    pub fn new(width: u32, height: u32) -> Result<Self, ConfigError> {
        if width == 0 || height == 0 {
            return Err(ConfigError::InvalidImageDimensions);
        }

        Ok(Self {
            width,
            height,
            antialias: AntialiasOptions::Disabled,
        })
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    /// Use for smoothening rough edges and color differences.
    ///
    /// Specify samples per pixel (SPP). Specifying 0 will disable antialiasing.
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

impl Default for RenderOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Camera orientation and basis vectors.
#[derive(Clone, Copy, Debug)]
pub struct CameraPose {
    center: Point,
    u: UtVector,
    v: UtVector,
    w: UtVector,
}

impl CameraPose {
    pub fn look_at(look_from: Point, look_at: Point, up: Vector) -> Result<Self, ConfigError> {
        if !look_from.is_finite() || !look_at.is_finite() || !up.is_finite() {
            return Err(ConfigError::NonFinitePose);
        }

        let view_direction = look_from - look_at;
        if view_direction.len_squared() <= 1e-12 {
            return Err(ConfigError::DegenerateViewDirection);
        }
        let w = view_direction.unit();

        let u_direction = up.cross(&w);
        if u_direction.len_squared() <= 1e-12 {
            return Err(ConfigError::UpVectorParallelToView);
        }

        let u = u_direction.unit();
        let v = w.cross(&u).unit();

        Ok(Self {
            center: look_from,
            u,
            v,
            w,
        })
    }
}

/// Perspective projection settings for a camera.
#[derive(Clone, Copy, Debug)]
pub struct PerspectiveProjection {
    vfov: f64,
}

impl PerspectiveProjection {
    pub fn new(vfov_degrees: f64) -> Result<Self, ConfigError> {
        if !vfov_degrees.is_finite() || !(0.0..180.0).contains(&vfov_degrees) {
            return Err(ConfigError::InvalidFieldOfView);
        }

        Ok(Self { vfov: vfov_degrees })
    }
}

/// Optical camera model settings.
#[derive(Clone, Copy, Debug)]
pub enum CameraModel {
    Pinhole {
        focus_dist: f64,
    },
    ThinLens {
        focus_dist: f64,
        defocus_angle_degrees: f64,
    },
}

impl CameraModel {
    pub fn pinhole(focus_dist: f64) -> Result<Self, ConfigError> {
        validate_focus_dist(focus_dist)?;
        Ok(Self::Pinhole { focus_dist })
    }

    pub fn thin_lens(focus_dist: f64, defocus_angle_degrees: f64) -> Result<Self, ConfigError> {
        validate_focus_dist(focus_dist)?;
        validate_defocus_angle(defocus_angle_degrees)?;
        Ok(Self::ThinLens {
            focus_dist,
            defocus_angle_degrees,
        })
    }

    fn focus_dist(self) -> f64 {
        match self {
            Self::Pinhole { focus_dist } | Self::ThinLens { focus_dist, .. } => focus_dist,
        }
    }

    fn defocus_angle(self) -> f64 {
        match self {
            Self::Pinhole { .. } => 0.0,
            Self::ThinLens {
                defocus_angle_degrees,
                ..
            } => defocus_angle_degrees,
        }
    }

    fn uses_defocus(self) -> bool {
        matches!(self, Self::ThinLens { .. })
    }
}

/// Complete camera input configuration.
#[derive(Clone, Debug)]
pub struct CameraConfig {
    pose: CameraPose,
    image: ImageOptions,
    projection: PerspectiveProjection,
    model: CameraModel,
    render: RenderOptions,
}

impl CameraConfig {
    pub fn new(
        pose: CameraPose,
        image: ImageOptions,
        projection: PerspectiveProjection,
        model: CameraModel,
    ) -> Self {
        Self {
            pose,
            image,
            projection,
            model,
            render: RenderOptions::default(),
        }
    }

    pub fn render(mut self, render: RenderOptions) -> Self {
        self.render = render;
        self
    }
}

#[derive(Clone, Debug)]
pub struct Camera {
    pose: CameraPose,
    projection: PerspectiveProjection,
    model: CameraModel,
    viewport_u: Vector,
    viewport_v: Vector,
    pixel_delta_u: Vector,
    pixel_delta_v: Vector,
    defocus_disk_u: Vector,
    defocus_disk_v: Vector,
    viewport_upper_left: Point,
    pixel00_loc: Point,
    pixel_samples_scale: Option<f64>,
    image_options: ImageOptions,
    render_options: RenderOptions,
}

impl Camera {
    pub fn new(config: CameraConfig) -> Self {
        let mut camera = Self {
            pose: config.pose,
            projection: config.projection,
            model: config.model,
            viewport_u: Vector::new(0.0, 0.0, 0.0),
            viewport_v: Vector::new(0.0, 0.0, 0.0),
            pixel_delta_u: Vector::new(0.0, 0.0, 0.0),
            pixel_delta_v: Vector::new(0.0, 0.0, 0.0),
            defocus_disk_u: Vector::new(0.0, 0.0, 0.0),
            defocus_disk_v: Vector::new(0.0, 0.0, 0.0),
            viewport_upper_left: Point::new(0.0, 0.0, 0.0),
            pixel00_loc: Point::new(0.0, 0.0, 0.0),
            pixel_samples_scale: None,
            image_options: config.image,
            render_options: config.render,
        };
        camera.recompute_geometry();
        camera
    }

    pub fn set_image_options(&mut self, image_options: ImageOptions) {
        self.image_options = image_options;
        self.recompute_geometry();
    }

    pub fn set_render_options(&mut self, render_options: RenderOptions) {
        self.render_options = render_options;
    }

    pub fn render<T: AsRef<Path>>(
        &self,
        world: &HittableList,
        path: T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        self.write_ppm_p3_header(&mut file)?;

        use ParallelOptions::*;
        match self.render_options.parallel {
            AllAtOnce => self.render_parallel_all(world, &mut file)?,
            ByRows => self.render_parallel_by_rows(world, &mut file)?,
            Series => self.render_sequential(world, &mut file)?,
        };

        Ok(())
    }

    pub fn render_in_memory(&self, world: &HittableList) -> Vec<Color> {
        use ParallelOptions::*;
        match self.render_options.parallel {
            AllAtOnce => {
                let mut pixels = vec![
                    Color::new(0.0, 0.0, 0.0);
                    (self.image_options.height * self.image_options.width)
                        as usize
                ];

                pixels.par_iter_mut().enumerate().for_each(|(i, v)| {
                    let x = (i as u32) % self.image_options.width;
                    let y = (i as u32) / self.image_options.width;
                    *v = self.pixel_color_at(world, x, y);
                });

                pixels
            }
            ByRows => {
                let mut pixels = Vec::with_capacity(
                    (self.image_options.height * self.image_options.width) as usize,
                );

                for j in 0..self.image_options.height {
                    let row_pixels: Vec<_> = (0..self.image_options.width)
                        .into_par_iter()
                        .map(|i| self.pixel_color_at(world, i, j))
                        .collect();
                    pixels.extend(row_pixels);
                }

                pixels
            }
            Series => {
                let mut pixels = Vec::with_capacity(
                    (self.image_options.height * self.image_options.width) as usize,
                );

                for j in 0..self.image_options.height {
                    for i in 0..self.image_options.width {
                        pixels.push(self.pixel_color_at(world, i, j));
                    }
                }

                pixels
            }
        }
    }

    fn recompute_geometry(&mut self) {
        let theta = (self.projection.vfov / 180.0) * std::f64::consts::PI;
        let h = (theta / 2.0).tan();
        let focus_dist = self.model.focus_dist();
        let viewport_height = 2.0 * h * focus_dist;
        let viewport_width = viewport_height * self.image_options.aspect_ratio();

        self.viewport_u = self.pose.u.inner() * viewport_width;
        self.viewport_v = -self.pose.v.inner() * viewport_height;
        self.pixel_delta_u = self.viewport_u / self.image_options.width as f64;
        self.pixel_delta_v = self.viewport_v / self.image_options.height as f64;
        self.viewport_upper_left = self.pose.center
            - (self.pose.w.inner() * focus_dist)
            - self.viewport_u / 2.0
            - self.viewport_v / 2.0;
        self.pixel00_loc =
            self.viewport_upper_left + (self.pixel_delta_u + self.pixel_delta_v) * 0.5;

        let defocus_radius =
            focus_dist * (utils::degrees_to_radians(self.model.defocus_angle() / 2.0)).tan();
        self.defocus_disk_u = self.pose.u.inner() * defocus_radius;
        self.defocus_disk_v = self.pose.v.inner() * defocus_radius;

        self.pixel_samples_scale = match self.image_options.antialias {
            AntialiasOptions::Disabled => None,
            AntialiasOptions::Enabled(samples_per_pixel) => Some(1.0 / samples_per_pixel as f64),
        };
    }

    /// Internal function to write P3 PPM header.
    fn write_ppm_p3_header(&self, file: &mut fs::File) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(file, "P3")?;
        writeln!(
            file,
            "{} {}",
            self.image_options.width, self.image_options.height
        )?;
        writeln!(file, "255")?;

        Ok(())
    }

    fn render_parallel_all(
        &self,
        world: &HittableList,
        file: &mut fs::File,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut pixels = vec![
            Color::new(0.0, 0.0, 0.0);
            (self.image_options.height * self.image_options.width) as usize
        ];

        pixels.par_iter_mut().enumerate().for_each(|(i, v)| {
            let x = (i as u32) % self.image_options.width;
            let y = (i as u32) / self.image_options.width;
            *v = self.pixel_color_at(world, x, y);
        });

        info!("Finished calculations!");

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

    fn render_parallel_by_rows(
        &self,
        world: &HittableList,
        file: &mut fs::File,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for j in 0..self.image_options.height {
            info!("Scanlines remaining: {}", self.image_options.height - j);
            io::stdout().flush().unwrap();

            let row_pixels: Vec<_> = (0..self.image_options.width)
                .into_par_iter()
                .map(|i| self.pixel_color_at(world, i, j))
                .collect();

            for pixel_color in row_pixels {
                writeln!(file, "{}", pixel_color)?;
            }
        }
        Ok(())
    }

    fn render_sequential(
        &self,
        world: &HittableList,
        file: &mut fs::File,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for j in 0..self.image_options.height {
            info!("Scanlines remaining: {}", self.image_options.height - j);
            io::stdout().flush().unwrap();
            for i in 0..self.image_options.width {
                let pixel_color = self.pixel_color_at(world, i, j);
                writeln!(file, "{}", pixel_color)?;
            }
        }

        Ok(())
    }

    fn pixel_color_at(&self, world: &HittableList, i: u32, j: u32) -> Color {
        let mut pixel_color = Color::new(0.0, 0.0, 0.0);

        use AntialiasOptions::*;
        match self.image_options.antialias {
            Disabled => {
                let pixel_center = self.get_pixel_center_coordinates(i, j);
                let ray_origin = if self.model.uses_defocus() {
                    self.defocus_disk_sample()
                } else {
                    self.pose.center
                };
                let ray_direction = (pixel_center - ray_origin).unit();
                let r = Ray::new(&ray_origin, ray_direction);
                pixel_color += r.color(world, 50);
            }
            Enabled(samples_per_pixel) => {
                for _ in 0..samples_per_pixel {
                    let (ray_origin, ray_dir) = self.get_antialiasing_ray_components(i, j);
                    let r = Ray::new(&ray_origin, ray_dir);
                    pixel_color += r.color(world, 50) * self.pixel_samples_scale.unwrap();
                }
            }
        }
        pixel_color
    }

    fn get_pixel_center_coordinates(&self, i: u32, j: u32) -> Point {
        self.pixel00_loc + (self.pixel_delta_u * i as f64) + (self.pixel_delta_v * j as f64)
    }

    fn get_antialiasing_ray_components(&self, i: u32, j: u32) -> (Point, UtVector) {
        let offset = Self::sample_square();
        let point_to = self.pixel00_loc
            + (self.pixel_delta_u * (i as f64 + offset.x()))
            + (self.pixel_delta_v * (j as f64 + offset.y()));
        let ray_origin = if self.model.uses_defocus() {
            self.defocus_disk_sample()
        } else {
            self.pose.center
        };
        let ray_direction = (point_to - ray_origin).unit();
        (ray_origin, ray_direction)
    }

    fn sample_square() -> Vector {
        Vector::new(
            rand::random_range(-0.5, 0.5),
            rand::random_range(-0.5, 0.5),
            0.0,
        )
    }

    fn defocus_disk_sample(&self) -> Point {
        let p = Vector::random_in_unit_disk();
        self.pose.center + (self.defocus_disk_u * p.x()) + (self.defocus_disk_v * p.y())
    }
}

fn validate_focus_dist(focus_dist: f64) -> Result<(), ConfigError> {
    if !focus_dist.is_finite() || focus_dist <= 0.0 {
        return Err(ConfigError::InvalidFocusDistance);
    }
    Ok(())
}

fn validate_defocus_angle(angle_degrees: f64) -> Result<(), ConfigError> {
    if !angle_degrees.is_finite() || !(0.0..180.0).contains(&angle_degrees) {
        return Err(ConfigError::InvalidDefocusAngle);
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("image width and height must both be greater than zero")]
    InvalidImageDimensions,
    #[error("vertical field of view must be finite and between 0 and 180 degrees")]
    InvalidFieldOfView,
    #[error("focus distance must be finite and greater than zero")]
    InvalidFocusDistance,
    #[error("defocus angle must be finite and between 0 and 180 degrees")]
    InvalidDefocusAngle,
    #[error("look_from, look_at, and up must all be finite vectors")]
    NonFinitePose,
    #[error("look_from and look_at must not be the same point")]
    DegenerateViewDirection,
    #[error("up vector must not be parallel to the view direction")]
    UpVectorParallelToView,
}
