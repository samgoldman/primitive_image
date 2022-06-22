use crate::point::PrimitivePoint;
use crate::primitive_image::PrimitiveImage;
use crate::shape::{RandomShape, Shape};
use crate::utilities::{clamp, get_rng, rgb_to_hex, rotate_point};
use image::ImageBuffer;
use image::Pixel;
use image::Rgba;
use rand::Rng;
use rand_distr::Normal;
use std::cmp::max;

const MAXIMUM_MUTATION_ATTEMPTS: u32 = 100_000;

#[derive(Debug, Copy, Clone)]
pub struct Ellipse {
    pub color: image::Rgba<u8>,
    center: PrimitivePoint,
    a: i32,
    b: i32,
    angle: u32, // In degrees
}

impl Ellipse {
    ///
    ///
    ///
    fn is_valid(&self, width: u32, height: u32) -> bool {
        (self.a as f64) < (width as f64 * 0.1) && (self.b as f64) < (height as f64 * 0.1)
    }

    fn un_rotated_contains_pixel(&self, x: i32, y: i32) -> bool {
        ((x - self.center.x) * (x - self.center.x)) as f64 / (self.a * self.a) as f64
            + ((y - self.center.y) * (y - self.center.y)) as f64 / (self.b * self.b) as f64
            <= 1.0
    }
}

impl RandomShape for Ellipse {
    ///
    /// Generate a random Triangle within the bounds given
    /// `border_extension` is the maximum distance outside of the border a triangle is allowed to go
    ///     It must be >= 1
    ///
    fn random(width: u32, height: u32, _border_extension: i32, seed: u64) -> Box<dyn Shape> {
        let mut rng = get_rng(seed);

        let center = PrimitivePoint::random_point(width, height, seed);
        let a = rng.gen_range(1..max(width as i32, height as i32) / 10);
        let b = rng.gen_range(1..max(width as i32, height as i32) / 10);
        let angle = rng.gen_range(0..360);

        let mut ellipse = Ellipse {
            center,
            a,
            b,
            angle,
            color: Rgba([0, 0, 0, 128]),
        };
        ellipse.mutate(width, height, seed);

        Box::new(ellipse)
    }
}

impl Shape for Ellipse {
    fn mutate(&mut self, width: u32, height: u32, seed: u64) {
        let mut rng = get_rng(seed);
        let normal = Normal::new(0.0, 5.0).unwrap();

        let mut i = 0;
        loop {
            i += 1;
            let r = rng.gen_range(0..4);

            match r {
                0 => self.center.mutate(width, height, seed),
                1 => {
                    self.a = clamp(
                        self.a as i32 + (rng.sample(normal) as i32),
                        1,
                        max(width, height) as i32,
                    )
                }
                2 => {
                    self.b = clamp(
                        self.b as i32 + (rng.sample(normal) as i32),
                        1,
                        max(width, height) as i32,
                    )
                }
                3 => {
                    self.angle =
                        clamp(self.angle as i32 + (rng.sample(normal) as i32), 0, 359) as u32
                }
                _ => {}
            }

            if self.is_valid(width, height) {
                break;
            }
            if i > MAXIMUM_MUTATION_ATTEMPTS {
                panic!("Ellipse: Too many mutation loops!");
            }
        }
    }

    fn get_pixels(&self) -> Vec<PrimitivePoint> {
        let min_x = self.center.x - (self.a as i32);
        let min_y = self.center.y - (self.b as i32);
        let max_x = self.center.x + (self.a as i32);
        let max_y = self.center.y + (self.b as i32);

        let mut pixels = vec![];

        for x in min_x..max_x {
            for y in min_y..max_y {
                if self.un_rotated_contains_pixel(x, y) {
                    pixels.push(PrimitivePoint::new(x, y));
                }
            }
        }

        for pixel in pixels.iter_mut() {
            rotate_point(pixel, self.center, self.angle);
        }
        pixels
    }

    fn as_svg(&self, scale: f64) -> String {
        let new_center = PrimitivePoint::new(
            (self.center.x as f64 * scale) as i32,
            (self.center.y as f64 * scale) as i32,
        );

        format!("<ellipse fill=\"{}\" fill-opacity=\"{:.5}\" cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" transform=\"rotate({} {} {})\"/>",
                rgb_to_hex(self.color),
                self.color.0[3] as f64 / 255.0,
                new_center.x, new_center.y,
                self.a as f64 * scale, self.b as f64 * scale,
                -(self.angle as i32), new_center.x, new_center.y)
    }

    //noinspection RsTypeCheck
    fn paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = image.clone();

        let pixels = self.get_pixels();

        for pixel in pixels.iter() {
            if pixel.x > 0 && pixel.y > 0 && pixel.x < width as i32 && pixel.y < height as i32 {
                let pix = output.get_pixel_mut(pixel.x as u32, pixel.y as u32);
                pix.blend(&self.color);
            }
        }

        output
    }

    //noinspection RsTypeCheck
    fn scaled_paint_on(
        &self,
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        scale: f64,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let scaled_self = Ellipse {
            center: PrimitivePoint::new(
                (self.center.x as f64 * scale) as i32,
                (self.center.y as f64 * scale) as i32,
            ),
            a: (self.a as f64 * scale) as i32,
            b: (self.b as f64 * scale) as i32,
            color: self.color,
            angle: self.angle,
        };

        scaled_self.paint_on(image)
    }

    fn set_color_using(&mut self, image: &PrimitiveImage) {
        self.color = image.target_average_color_in_shape(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_un_rotated_contains_pixel() {
        let center = PrimitivePoint::new(2, 2);
        let a = 2;
        let b = 2;
        let angle = 0;
        let ellipse = Ellipse {
            center,
            a,
            b,
            angle,
            color: Rgba([0, 0, 0, 0]),
        };
        assert_eq!(ellipse.un_rotated_contains_pixel(2, 2), true);
        assert_eq!(ellipse.un_rotated_contains_pixel(0, 0), false);

        let center = PrimitivePoint::new(2, 2);
        let a = 10;
        let b = 10;
        let angle = 0;
        let ellipse = Ellipse {
            center,
            a,
            b,
            angle,
            color: Rgba([0, 0, 0, 0]),
        };
        assert_eq!(ellipse.un_rotated_contains_pixel(2, -8), true);
        assert_eq!(ellipse.un_rotated_contains_pixel(12, -8), false);
        assert_eq!(ellipse.un_rotated_contains_pixel(11, -7), false);
    }
}
