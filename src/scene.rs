//! Camera configuration and rendering API.
//!
//! The public flow is:
//! 1. Build validated camera inputs such as [`CameraPose`], [`PerspectiveProjection`],
//!    [`CameraModel`], and [`ImageOptions`].
//! 2. Assemble them into a [`CameraConfig`].
//! 3. Build a reusable [`Camera`].
//! 4. Render that camera against any world implementing [`Hittable`].

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

use log::info;
use rayon::prelude::*;
use thiserror::Error;

use crate::color::Color;
use crate::objects::Hittable;
use crate::ray::Ray;
use crate::utils::{self, rand};
use crate::vector::{Point, UtVector, Vector};

/// Output image dimensions and sampling settings used by a [`Camera`].
///
/// Dimensions are validated up front so a camera can safely accept fresh
/// [`ImageOptions`] later through [`Camera::set_image_options`] without needing
/// to return an error.
#[derive(Copy, Clone, Debug)]
pub struct ImageOptions {
    width: u32,
    height: u32,
    antialias: AntialiasOptions,
}

/// Internal antialiasing mode for [`ImageOptions`].
#[derive(Debug, Clone, Copy)]
enum AntialiasOptions {
    Disabled,
    Enabled(u32),
}

impl ImageOptions {
    /// Create a new set of image options.
    ///
    /// Returns [`ConfigError::InvalidImageDimensions`] when either dimension is 0.
    ///
    /// ```rs
    /// # use ray_tracing_rs::ImageOptions;
    /// let image = ImageOptions::new(1200, 675)?.antialias(50);
    /// # Ok::<(), ray_tracing_rs::ConfigError>(())
    /// ```
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

    /// Return the image aspect ratio as `width / height`.
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    /// Configure antialiasing samples per pixel.
    ///
    /// Specifying 0 disables antialiasing, which is also the default.
    ///
    /// ```rs
    /// # use ray_tracing_rs::ImageOptions;
    /// let image = ImageOptions::new(800, 450)?.antialias(10);
    /// # Ok::<(), ray_tracing_rs::ConfigError>(())
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

/// Render-time scheduling options.
///
/// These do not change the rays a camera emits, only how the pixel work is
/// scheduled and written.
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
    /// Create render options using [`ParallelOptions::ByRows`].
    pub fn new() -> Self {
        Self {
            parallel: ParallelOptions::ByRows,
        }
    }

    /// Override the rendering strategy.
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

/// Validated camera orientation and basis vectors.
///
/// This stores the camera position along with the orthonormal basis derived
/// from `look_from`, `look_at`, and `up`.
#[derive(Clone, Copy, Debug)]
pub struct CameraPose {
    center: Point,
    u: UtVector,
    v: UtVector,
    w: UtVector,
}

impl CameraPose {
    /// Build a camera pose from the usual "look at" inputs.
    ///
    /// This validates the geometric constraints that depend on several inputs at
    /// once: the camera position must differ from the target, and `up` must not
    /// be parallel to the viewing direction.
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

/// Validated perspective projection settings for a camera.
#[derive(Clone, Copy, Debug)]
pub struct PerspectiveProjection {
    vfov: f64,
}

impl PerspectiveProjection {
    /// Create a perspective projection from a vertical field of view in degrees.
    ///
    /// Valid values are finite numbers strictly between 0 and 180.
    pub fn new(vfov_degrees: f64) -> Result<Self, ConfigError> {
        if !vfov_degrees.is_finite() || vfov_degrees <= 0.0 || vfov_degrees >= 180.0 {
            return Err(ConfigError::InvalidFieldOfView);
        }

        Ok(Self { vfov: vfov_degrees })
    }
}

/// Optical camera model settings.
///
/// [`CameraModel::Pinhole`] disables depth-of-field blur. [`CameraModel::ThinLens`]
/// enables it by sampling a defocus disk.
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
    /// Create a pinhole camera model with a validated focus distance.
    pub fn pinhole(focus_dist: f64) -> Result<Self, ConfigError> {
        validate_focus_dist(focus_dist)?;
        Ok(Self::Pinhole { focus_dist })
    }

    /// Create a thin-lens camera model with depth-of-field blur.
    ///
    /// Both `focus_dist` and `defocus_angle_degrees` are validated.
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

/// Complete validated camera input configuration.
///
/// [`CameraConfig`] groups the stable user-controlled camera inputs. The
/// expensive, derived geometry lives on [`Camera`] itself, while render-time
/// scheduling stays separate.
#[derive(Clone, Debug)]
pub struct CameraConfig {
    pose: CameraPose,
    image: ImageOptions,
    projection: PerspectiveProjection,
    model: CameraModel,
}

impl CameraConfig {
    /// Assemble the validated inputs required to build a [`Camera`].
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
        }
    }
}

/// A reusable camera with precomputed geometry.
///
/// A [`Camera`] is independent from any particular scene. The same camera can
/// render multiple worlds, and multiple cameras can render the same world.
/// Render scheduling is intentionally not stored here.
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
}

impl Camera {
    /// Build a camera from a fully validated [`CameraConfig`].
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
        };
        camera.recompute_geometry();
        camera
    }

    /// Replace the image settings and recompute the derived camera geometry.
    pub fn set_image_options(&mut self, image_options: ImageOptions) {
        self.image_options = image_options;
        self.recompute_geometry();
    }

    /// Render the camera to a P3 PPM file using default render options.
    ///
    /// The scene is passed in explicitly so camera configuration stays separate
    /// from world ownership.
    pub fn render<T: AsRef<Path>>(
        &self,
        world: &dyn Hittable,
        path: T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.render_with_options(world, path, &RenderOptions::default())
    }

    /// Render the camera to a P3 PPM file using an explicit render policy.
    pub fn render_with_options<T: AsRef<Path>>(
        &self,
        world: &dyn Hittable,
        path: T,
        render_options: &RenderOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        self.write_ppm_p3_header(&mut file)?;

        use ParallelOptions::*;
        match render_options.parallel {
            AllAtOnce => self.render_parallel_all(world, &mut file)?,
            ByRows => self.render_parallel_by_rows(world, &mut file)?,
            Series => self.render_sequential(world, &mut file)?,
        };

        Ok(())
    }

    /// Render the camera into memory without writing a file using default render options.
    ///
    /// This is useful for tests and benchmarks that want to measure ray
    /// generation and shading without including file I/O.
    pub fn render_in_memory(&self, world: &dyn Hittable) -> Vec<Color> {
        self.render_in_memory_with_options(world, &RenderOptions::default())
    }

    /// Render the camera into memory using an explicit render policy.
    pub fn render_in_memory_with_options(
        &self,
        world: &dyn Hittable,
        render_options: &RenderOptions,
    ) -> Vec<Color> {
        use ParallelOptions::*;
        match render_options.parallel {
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
        world: &dyn Hittable,
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
        world: &dyn Hittable,
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
        world: &dyn Hittable,
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

    fn pixel_color_at(&self, world: &dyn Hittable, i: u32, j: u32) -> Color {
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
    if !angle_degrees.is_finite() || angle_degrees <= 0.0 || angle_degrees >= 180.0 {
        return Err(ConfigError::InvalidDefocusAngle);
    }
    Ok(())
}

/// Errors returned while validating public camera configuration inputs.
#[derive(Error, Debug, PartialEq, Eq)]
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
