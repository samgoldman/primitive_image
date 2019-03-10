use crate::shape::{Shape, RandomShape};
use crate::point::PrimitivePoint;
use crate::primitive_image::PrimitiveImage;
use image::Rgba;
use std::cmp::max;
use rand;
use rand::Rng;
use image::ImageBuffer;
use imageproc::drawing::draw_filled_ellipse;
use imageproc::affine::rotate;
use imageproc::affine::Interpolation::Nearest;
use image::imageops::overlay;
use crate::utilities::{get_rng, clamp, radians, rgb_to_hex, rotate_point};
use rand::distributions::Distribution;
use rand::distributions::Normal;


const MAXIMUM_MUTATION_ATTEMPTS: u32 = 100_000;

#[derive(Debug, Copy, Clone)]
pub struct Ellipse {
    pub color: image::Rgba<u8>,
    center: PrimitivePoint,
    a: u32,
    b: u32,
    angle: u32 // In degrees
}

impl Ellipse {

    ///
    /// Determine if this rectangle is valid
    ///
    fn is_valid(&self) -> bool {
        true
    }

    fn un_rotated_contains_pixel(&self, x: i32, y: i32) -> bool {
        ((x - self.center.x)*(x - self.center.x))/(self.a*self.a) as i32 + ((y - self.center.y)*(y - self.center.y))/(self.b*self.b) as i32 <= 1
    }
}

impl RandomShape for Ellipse {

    ///
    /// Generate a random Triangle within the bounds given
    /// `border_extension` is the maximum distance outside of the border a triangle is allowed to go
    ///     It must be >= 1
    ///
    fn random(width: u32, height: u32, _border_extension: i32, seed: u64) -> Box<Shape> {
        let mut rng = get_rng(seed);

        let center = PrimitivePoint::random_point(width, height, seed);
        let a = rng.gen_range(5, max(width, height) / 2);
        let b = rng.gen_range(5, max(width, height) / 2);
        let angle = rng.gen_range(0, 180);

        let mut ellipse = Ellipse{center, a, b, angle, color: Rgba([0, 0, 0, 128])};
        ellipse.mutate(width, height, seed);

        Box::new(ellipse)
    }
}

impl Shape for Ellipse {

    fn mutate(&mut self, width: u32, height: u32, seed: u64) {
        let mut rng = get_rng(seed);
        let normal = Normal::new(0.0, 16.0);


        let mut i = 0;
        loop {
            i += 1;
            let r = rng.gen_range(0, 4);

            match r {
                0 => self.center.mutate(width, height, seed),
                1 => self.a = clamp(self.a as i32 + (normal.sample(&mut rng) as i32), 5, max(width, height) as i32) as u32,
                2 => self.b = clamp(self.b as i32 + (normal.sample(&mut rng) as i32), 5, max(width, height) as i32) as u32,
                3 => self.angle = rng.gen_range(0, 180),
                _ => {}
            }

            if self.is_valid() {
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

        for i in 0..pixels.len() {
            rotate_point(&mut pixels[i], self.center, self.angle);
        }
        pixels
    }

    fn as_svg(&self, scale: f64) -> String {
        let new_center = PrimitivePoint::new((self.center.x as f64 * scale) as i32, (self.center.y as f64 * scale) as i32);

        let min_x = new_center.x - ((self.a as f64 * scale) as i32);
        let min_y = new_center.y - ((self.b as f64 * scale) as i32);

        let p1 = PrimitivePoint::new(min_x, min_y);
        format!("<ellipse fill=\"{}\" fill-opacity=\"{:.5}\" cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" transform=\"rotate({} {} {})\"/>",
                rgb_to_hex(self.color),
                self.color.data[3] as f64 / 255.0,
                p1.x, p1.y,
                self.a as f64 * scale, self.b as f64 * scale,
                self.angle, p1.x as f64 + self.a as f64 * scale, p1.y as f64 + self.b as f64 * scale)
    }

    // Suppress intellij inspection for E0308 (false positive)
    //noinspection RsTypeCheck
    fn paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();

        let mut ell_image: ImageBuffer<Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_pixel(width as u32, height as u32, image::Rgba([0, 0, 0, 0]));
        let mut output = image.clone();

        ell_image = draw_filled_ellipse(&ell_image, (self.center.x, self.center.y), self.a as i32, self.b as i32, self.color);
        ell_image = rotate(&ell_image, (self.center.x as f32, self.center.y as f32), -1.0*radians(self.angle as f64) as f32, Nearest);

        overlay(&mut output, &ell_image, 0, 0);

        output
    }

    // Suppress intellij inspection for E0308 (false positive)
    //noinspection RsTypeCheck
    fn scaled_paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>, scale: f64) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();

        let mut ell_image: ImageBuffer<Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_pixel(width as u32, height as u32, image::Rgba([0, 0, 0, 0]));
        let mut output = image.clone();

        ell_image = draw_filled_ellipse(&ell_image, ((self.center.x as f64 * scale) as i32, (self.center.y as f64 * scale) as i32), (self.a as f64 * scale) as i32, (self.b as f64 * scale) as i32, self.color);
        ell_image = rotate(&ell_image, ((self.center.x as f64 * scale) as f32, (self.center.y as f64 * scale) as f32), -1.0*radians(self.angle as f64) as f32, Nearest);

        overlay(&mut output, &ell_image, 0, 0);

        output
    }

    fn set_color_using(&mut self, image: &PrimitiveImage) {
        self.color = image.target_average_color_in_shape(&Box::new(*self));
    }
}