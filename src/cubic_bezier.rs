use crate::shape::{Shape, RandomShape};
use crate::point::PrimitivePoint;
use crate::primitive_image::PrimitiveImage;
use image::Rgba;
use imageproc::drawing::BresenhamLineIter;
use rand;
use rand::Rng;
use image::ImageBuffer;
use imageproc::drawing::draw_cubic_bezier_curve;
use image::imageops::overlay;
use crate::utilities::get_rng;
use crate::utilities::rgb_to_hex;

const MAXIMUM_MUTATION_ATTEMPTS: u32 = 100_000;

#[derive(Debug, Copy, Clone)]
pub struct CubicBezier {
    pub color: image::Rgba<u8>,
    pub start: PrimitivePoint,
    pub control1: PrimitivePoint,
    pub control2: PrimitivePoint,
    pub end: PrimitivePoint
}

impl CubicBezier {
    fn new(start: PrimitivePoint, end: PrimitivePoint, control1: PrimitivePoint, control2: PrimitivePoint) -> Box<CubicBezier> {
        Box::new(CubicBezier {color: Rgba([0, 0, 0, 128]), start, end, control1, control2})
    }

    /// Currently no validation for CubicBezier is required, so this always returns true
    fn is_valid(&self) -> bool {
        true
    }
}

impl RandomShape for CubicBezier {
    fn random(width: u32, height: u32, border_extension: i32, seed: u64) -> Box<Shape> {
        let start = PrimitivePoint::random_point(width, height, seed);
        let c1 = start.random_point_in_radius(border_extension, seed);
        let c2 = start.random_point_in_radius(border_extension, seed);
        let end = start.random_point_in_radius(border_extension, seed);

        let mut bezier = CubicBezier::new(start, end, c1, c2);
        bezier.mutate(width, height, seed);

        bezier
    }
}

impl Shape for CubicBezier {
    fn mutate(&mut self, width: u32, height: u32, seed: u64) {
        let mut rng = get_rng(seed);

        let mut i = 0;
        loop {
            i += 1;
            let r = rng.gen_range(0, 4);

            match r {
                0 => self.start.mutate(width, height, seed),
                1 => self.end.mutate(width, height, seed),
                2 => self.control1.mutate(width, height, seed),
                3 => self.control2.mutate(width, height, seed),
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
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;
            let x = (self.start.x as f32 * mt3) + (3.0 * self.control1.x as f32 * mt2 * t) + (3.0 * self.control2.x as f32 * mt * t2) + (self.end.x as f32 * t3);
            let y = (self.start.y as f32 * mt3) + (3.0 * self.control1.y as f32 * mt2 * t) + (3.0 * self.control2.y as f32 * mt * t2) + (self.end.y as f32 * t3);
            (x.round(), y.round()) // round to nearest pixel, to avoid ugly line artifacts
        };

        let distance = |point_a: (f32, f32), point_b: (f32, f32)| {
            ((point_a.0 - point_b.0).powi(2) + (point_a.1 - point_b.1).powi(2)).sqrt()
        };

        // Approximate curve's length by adding distance between control points.
        let curve_length_bound: f32 = distance((self.start.x as f32, self.start.y as f32), (self.control1.x as f32, self.control1.y as f32)) +
            distance((self.control1.x as f32, self.control1.y as f32), (self.control2.x as f32, self.control2.y as f32)) +
            distance((self.control2.x as f32, self.control2.y as f32), (self.end.x as f32, self.end.y as f32));

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

    fn as_svg(&self, scale: f64) -> String {
        format!("<path stroke=\"{}\" stroke-opacity=\"{}\" fill=\"none\" d=\"M{} {} C{} {}, {} {}, {} {}\" stroke-width=\"{}\" />",
                rgb_to_hex(self.color),
                self.color.data[3] as f64 / 255.0,
                (self.start.x as f64 * scale) as i32, (self.start.y as f64 * scale) as i32,
                (self.control1.x as f64 * scale) as i32, (self.control1.y as f64 * scale) as i32,
                (self.control2.x as f64 * scale) as i32, (self.control2.y as f64 * scale) as i32,
                (self.end.x as f64 * scale) as i32, (self.end.y as f64 * scale) as i32,
                scale/2.0)
    }

    // Suppress intellij inspection for E0308 (false positive)
    //noinspection RsTypeCheck
    fn paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();

        let mut tri_image: ImageBuffer<Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_pixel(width as u32, height as u32, image::Rgba([0, 0, 0, 0]));
        let mut output = image.clone();

        tri_image = draw_cubic_bezier_curve(&tri_image, (self.start.x as f32, self.start.y as f32), (self.end.x as f32, self.end.y as f32), (self.control1.x as f32, self.control1.y as f32), (self.control2.x as f32, self.control1.y as f32), self.color);

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
        let c1 = (self.control1.x as f32 * scale as f32, self.control1.y as f32 * scale as f32);
        let c2 = (self.control2.x as f32 * scale as f32, self.control2.y as f32 * scale as f32);


        curve_image = draw_cubic_bezier_curve(&curve_image, start, end, c1, c2, self.color);

        overlay(&mut output, &curve_image, 0, 0);

        output
    }

    fn set_color_using(&mut self, image: &PrimitiveImage) {
        self.color = image.target_average_color_in_shape(&Box::new(*self));
    }
}