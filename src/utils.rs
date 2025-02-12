use std::f64::consts::PI;

pub fn degrees_to_radians(degrees: f64) -> f64 {
    (degrees / 180.0) * PI
}

pub fn radians_to_degrees(radians: f64) -> f64 {
    (radians / PI) * 180.0
}

pub mod interval {
    pub struct Interval {
        pub min: f64,
        pub max: f64,
    }

    impl Interval {
        pub fn new_empty() -> Self {
            Self {
                min: f64::MIN,
                max: f64::MAX,
            }
        }

        pub fn new(min: f64, max: f64) -> Self {
            Self { min, max }
        }

        pub fn contains(&self, t: f64) -> bool {
            t > self.min && t < self.max
        }

        pub fn contains_inclusive(&self, t: f64) -> bool {
            t >= self.min && t <= self.max
        }

        pub fn size(&self) -> f64 {
            self.max - self.min
        }
    }
}
