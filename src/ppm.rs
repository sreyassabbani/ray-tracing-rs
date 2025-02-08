use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn diagonal_gradient<T: AsRef<Path>>(path: T) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    // P3 PPM header
    writeln!(file, "P3")?;
    let image_height = 25;
    let image_width = 25;
    writeln!(file, "{} {}", image_width, image_height)?;
    writeln!(file, "255")?; // The maximum color value for RGB channels in P3

    // Write the pixel data
    for i in 0..image_height {
        for j in 0..image_width {
            let r = (i as f64) / (image_height as f64); // Gradient along the height
            let g = (j as f64) / (image_width as f64); // Gradient along the width
            writeln!(
                file,
                "{} {} {}",
                (r * 255.0) as u8, // Red byte
                (g * 255.0) as u8, // Green byte
                0,                 // Blue byte
            )?;
        }
    }
    Ok(())
}
