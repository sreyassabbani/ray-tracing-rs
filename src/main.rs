use ray_tracing_rs::ppm;
use ray_tracing_rs::vector::{Point, Vector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    ppm::ppm("output.ppm")?;
    let v1 = Vector::from([3.0, 4.0]);
    println!("{}", v1.norm());
    let v2 = Vector::from([1.0, 1.0]);
    println!("{:?}", v1 + v2);
    println!(
        "{:?}",
        Point::from([0.0, 0.0, 1.0]).cross(&Point::from([0.0, 1.0, 0.0]))
    );
    Ok(())
}
