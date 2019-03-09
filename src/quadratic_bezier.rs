use crate::shape::{Shape, RandomShape};
use crate::point::PrimitivePoint;
use crate::primitive_image::PrimitiveImage;
use image::Rgba;
use imageproc::drawing::BresenhamLineIter;
use std::cmp::{min, max};
use rand;
use rand::Rng;
use image::ImageBuffer;
use imageproc::drawing::draw_cubic_bezier_curve;
use image::imageops::overlay;
use crate::utilities::get_rng;

const MAXIMUM_MUTATION_ATTEMPTS: u32 = 100_000;

#[derive(Debug, Copy, Clone)]
pub struct QuadraticBezier {
    pub color: image::Rgba<u8>,
    pub start: PrimitivePoint,
    pub control: PrimitivePoint,
    pub end: PrimitivePoint
}

impl QuadraticBezier {
    fn new(start: PrimitivePoint, end: PrimitivePoint, control: PrimitivePoint) -> Box<QuadraticBezier> {
        Box::new(QuadraticBezier {color: Rgba([0, 0, 0, 128]), start, end, control})
    }

    /// Currently no validation for CubicBezier is required, so this always returns true
    fn is_valid(&self) -> bool {
        let dx12 = self.start.x - self.control.x;
        let dy12 = self.start.y - self.control.y;
        let dx23 = self.control.x - self.end.x;
        let dy23 = self.control.y - self.end.y;
        let dx13 = self.start.x - self.end.x;
        let dy13 = self.start.y - self.end.y;
        let d12 = dx12*dx12 + dy12*dy12;
        let d23 = dx23*dx23 + dy23*dy23;
        let d13 = dx13*dx13 + dy13*dy13;

        d13 > d12 && d13 > d23
    }
}

impl RandomShape for QuadraticBezier {
    fn random(width: u32, height: u32, border_extension: i32, seed: u64) -> Box<Shape> {
        let start = PrimitivePoint::random_point(width, height, seed);
        let control = start.random_point_in_radius(border_extension, seed);
        let end = start.random_point_in_radius(border_extension, seed);

        let mut bezier = QuadraticBezier::new(start, end, control);
        bezier.mutate(width, height, seed);

        bezier
    }
}

impl Shape for QuadraticBezier {
    fn mutate(&mut self, width: u32, height: u32, seed: u64) {
        let mut rng = get_rng(seed);

        let mut i = 0;
        loop {
            i += 1;
            let r = rng.gen_range(0, 3);

            match r {
                0 => self.start.mutate(width, height, seed),
                1 => self.end.mutate(width, height, seed),
                2 => self.control.mutate(width, height, seed),
                _ => {}
            }

            if self.is_valid() {
                break;
            }
            if i > MAXIMUM_MUTATION_ATTEMPTS {
                panic!("Cubic Bezier: Too many mutation loops!");
            }
        }
    }

    fn get_pixels(&self) -> Vec<PrimitivePoint> {
        // Modified from Imageproc's `draw_cubic_bezier_curve_mut` function
        // Bezier Curve function from: https://pomax.github.io/bezierinfo/#control
        let cubic_bezier_curve = |t: f32| {
            let t2 = t * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let x = (self.start.x as f32 * mt2) + (2.0 * self.control.x as f32 * mt * t) + (self.end.x as f32 * t2);
            let y = (self.start.y as f32 * mt2) + (2.0 * self.control.y as f32 * mt * t) + (self.end.y as f32 * t2);
            (x.round(), y.round()) // round to nearest pixel, to avoid ugly line artifacts
        };

        let distance = |point_a: (f32, f32), point_b: (f32, f32)| {
            ((point_a.0 - point_b.0).powi(2) + (point_a.1 - point_b.1).powi(2)).sqrt()
        };

        // Approximate curve's length by adding distance between control points.
        let curve_length_bound: f32 = distance((self.start.x as f32, self.start.y as f32), (self.control.x as f32, self.control.y as f32)) +
            distance((self.control.x as f32, self.control.y as f32), (self.end.x as f32, self.end.y as f32));

        // Use hyperbola function to give shorter curves a bias in number of line segments.
        let num_segments: i32 = ((curve_length_bound.powi(2) + 800.0).sqrt() / 8.0) as i32;

        // Sample points along the curve and connect them with line segments.
        let t_interval = 1f32 / (num_segments as f32);
        let mut t1 = 0f32;

        let mut pixels = vec![];

        'outer:for i in 0..num_segments {
            let t2 = (i as f32 + 1.0) * t_interval;

            let line_iterator = BresenhamLineIter::new(cubic_bezier_curve(t1), cubic_bezier_curve(t2));

            for point in line_iterator {
                let x = point.0;
                let y = point.1;
                pixels.push(PrimitivePoint::new(x, y));
            }

            t1 = t2;
        }

        pixels
    }

    fn bounding_box(&self) -> [PrimitivePoint; 2] {
        [PrimitivePoint::new(min(self.start.x, min(self.control.x, self.end.x)),
                             min(self.start.y, min(self.control.y, self.end.y))),

            PrimitivePoint::new(max(self.start.x, max(self.control.x, self.end.x)),
                                max(self.start.y, max(self.control.y, self.end.y)))]
    }

    fn as_svg(&self, scale: f64) -> String {
        format!("<path stroke=\"#{:X}{:X}{:X}\" stroke-opacity=\"{}\" fill=\"none\" d=\"M{} {} Q{} {}, {} {}\" stroke-width=\"{}\" />",
                self.color.data[0], self.color.data[1], self.color.data[2],
                self.color.data[3] as f64 / 255.0,
                (self.start.x as f64 * scale) as i32, (self.start.y as f64 * scale) as i32,
                (self.control.x as f64 * scale) as i32, (self.control.y as f64 * scale) as i32,
                (self.end.x as f64 * scale) as i32, (self.end.y as f64 * scale) as i32,
                1.0)
    }

    // Suppress intellij inspection for E0308 (false positive)
    //noinspection RsTypeCheck
    fn paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();

        let mut tri_image: ImageBuffer<Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_pixel(width as u32, height as u32, image::Rgba([0, 0, 0, 0]));
        let mut output = image.clone();

        tri_image = draw_cubic_bezier_curve(&tri_image,
                                            (self.start.x as f32, self.start.y as f32),
                                            (self.end.x as f32, self.end.y as f32),
                                            ((self.control.x as f32 - self.start.x as f32) * 2.0/3.0,
                                                        (self.control.y as f32 - self.start.y as f32) * 2.0/3.0),
                                            ((self.control.x as f32 - self.end.x as f32) * 2.0/3.0,
                                                        (self.control.y as f32 - self.end.y as f32) * 2.0/3.0),
                                            self.color);

        overlay(&mut output, &tri_image, 0, 0);

        output
    }

    // Suppress intellij inspection for E0308 (false positive)
    //noinspection RsTypeCheck
    fn scaled_paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>, scale: f64) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();

        let mut curve_image: ImageBuffer<Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_pixel(width as u32, height as u32, image::Rgba([0, 0, 0, 0]));
        let mut output = image.clone();

        let start = (self.start.x as f32 * scale as f32, self.start.y as f32 * scale as f32);
        let end = (self.end.x as f32 * scale as f32, self.end.y as f32 * scale as f32);
        let c1 = ((self.control.x as f32 - self.start.x as f32) * 2.0/3.0 * scale as f32, (self.control.y as f32 - self.start.y as f32) * 2.0/3.0 * scale as f32);
        let c2 = ((self.control.x as f32 - self.end.x as f32) * 2.0/3.0 * scale as f32, (self.control.y as f32 - self.end.y as f32) * 2.0/3.0 * scale as f32);


        curve_image = draw_cubic_bezier_curve(&curve_image, start, end, c1, c2, self.color);

        overlay(&mut output, &curve_image, 0, 0);

        output
    }

    fn set_color_using(&mut self, image: &PrimitiveImage) {
        self.color = image.target_average_color_in_shape(&Box::new(*self));
    }
}