mod ppm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    ppm::diagonal_gradient("output.ppm")?;
    Ok(())
}
