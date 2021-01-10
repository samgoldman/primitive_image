use std::f64;
use std::cmp::{min, max};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::prelude::*;
use image::Rgba;
use crate::point::PrimitivePoint;

/// Return the square root of x
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

/// Return acos of x
pub fn acos(x: f64) -> f64 {
    x.acos()
}

/// Convert x from radians to degrees
pub fn degrees(x: f64) -> f64 {
    x * 180.0 / f64::consts::PI
}

pub fn radians(x: f64) -> f64 {
    x / 180.0 * f64::consts::PI
}

pub fn clamp(value: i32, min_v: i32, max_v: i32) -> i32 {
    max(min_v, min(max_v, value))
}

pub fn get_rng(seed: u64) -> StdRng {
    if seed != 0 {
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::seed_from_u64(get_time_seed())
    }
}

fn get_time_seed() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // Seed
    since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64
}

pub fn rgb_to_hex(color: Rgba<u8>) -> String {
    let data = color.0;
    format!("#{:02X}{:02X}{:02X}", data[0], data[1], data[2])
}

pub fn rotate_point(point: &mut PrimitivePoint, center: PrimitivePoint, angle: u32) {
    let cos_a = radians(angle as f64).cos();
    let sin_a = radians(angle as f64).sin();

    point.x -= center.x;
    point.y -= center.y;

    let new_x = point.x as f64 * cos_a - point.y as f64 * sin_a;
    let new_y = point.x as f64 * sin_a + point.y as f64 * cos_a;

    point.x = new_x as i32 + center.x;
    point.y = new_y as i32 + center.y;
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt() {
        assert_eq!(sqrt(4.0), 2.0);
    }

    #[test]
    fn test_degrees() {
        assert_eq!(degrees(f64::consts::PI), 180.0);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(10, -10, 20), 10);
        assert_eq!(clamp(-20, -10, 20), -10);
        assert_eq!(clamp(30, -10, 20), 20);
    }

    #[test]
    fn test_rgb_to_hex() {
        let result = rgb_to_hex(Rgba([0, 0, 0, 0]));
        let test: &str = result.as_ref();
        assert_eq!(test, "#000000");

        let result = rgb_to_hex(Rgba([255, 0, 0, 0]));
        let test: &str = result.as_ref();
        assert_eq!(test, "#FF0000");

        let result = rgb_to_hex(Rgba([0, 12, 0, 0]));
        let test: &str = result.as_ref();
        assert_eq!(test, "#000C00");
    }

    #[test]
    fn test_rotate_point() {
        let mut p = PrimitivePoint::new(20, 10);
        let center = PrimitivePoint::new(10, 10);
        let angle = 90;
        let expected = PrimitivePoint::new(10, 20);
        rotate_point(&mut p, center, angle);
        assert_eq!(p, expected);


        let mut p = PrimitivePoint::new(20, 10);
        let center = PrimitivePoint::new(10, 10);
        let angle = 180;
        let expected = PrimitivePoint::new(0, 10);
        rotate_point(&mut p, center, angle);
        assert_eq!(p, expected);
    }
}