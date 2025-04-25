use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use ray_tracing_rs::color::Color;
use ray_tracing_rs::materials::Lambertian;
use ray_tracing_rs::objects::Sphere;
use ray_tracing_rs::scene::{ParallelOptions, RenderOptions};
use ray_tracing_rs::vector::Vector;
use ray_tracing_rs::{Camera, HittableList, ImageOptions, Point};

use std::time::Duration;

fn basic_world(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic-world");

    // World setup
    let mut world = HittableList::new();
    let diffuse = Lambertian::new(Color::new(1.0, 0.5, 0.5));
    let sphere = Sphere::new(Point::new(0.0, 0.0, -1.0), 0.5, diffuse.clone());
    let ground = Sphere::new(Point::new(0.0, -100.5, -1.0), 100.0, diffuse);
    world.add(sphere).unwrap();
    world.add(ground).unwrap();

    let image = ImageOptions::new(16, 9);
    let vfov = 90.0;
    let look_from = Point::new(-0.0, 0.0, 0.0);
    let look_at = Point::new(0.0, 0.0, -1.0);
    let up = Vector::new(0.0, 1.0, 0.0);
    let mut camera = Camera::new(vfov, 10.0, look_from, look_at, up, image, world).unwrap();

    // Bench for different samples per pixel (SPP)
    // 0 SPP configures AntialiasOptions::Disabled
    for spp in [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100] {
        group.bench_with_input(
            BenchmarkId::new("parallel-compute-by-rows", spp),
            &spp,
            |b, &spp| {
                b.iter(|| {
                    let render_options = RenderOptions::new().parallel(ParallelOptions::ByRows);
                    let image_options = image.clone().antialias(spp);
                    camera.update_render_options(render_options);
                    camera.update_image_options(image_options);
                    camera.render("output.ppm")
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parallel-compute-at-once", spp),
            &spp,
            |b, &spp| {
                b.iter(|| {
                    let render_options = RenderOptions::new().parallel(ParallelOptions::AllAtOnce);
                    let image_options = image.clone().antialias(spp);
                    camera.update_render_options(render_options);
                    camera.update_image_options(image_options);
                    camera.render("output.ppm")
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("series-computation", spp),
            &spp,
            |b, &spp| {
                b.iter(|| {
                    let render_options = RenderOptions::new().parallel(ParallelOptions::Series);
                    let image_options = image.clone().antialias(spp);
                    camera.update_render_options(render_options);
                    camera.update_image_options(image_options);
                    camera.render("output.ppm")
                })
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(10));
    targets = basic_world
}
criterion_main!(benches);
