use crate::polygon::{Polygon, RandomPolygon};
use crate::point::PrimitivePoint;
use crate::primitive_image::PrimitiveImage;
use image::Rgba;
use imageproc::drawing::Point;
use std::cmp::{min, max};
use rand;
use rand::prelude::*;
use rand::Rng;
use image::ImageBuffer;
use imageproc::drawing::draw_convex_polygon;
use image::imageops::overlay;
use crate::utilities::get_time_seed;

const MINIMUM_DEGREES: f64 = 15.0;

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub color: image::Rgba<u8>,
    pub path: [PrimitivePoint; 3]
}

impl Triangle {
    fn get_drawing_points(&self) -> Vec<Point<i32>> {
        vec![self.path[0].to_drawing_point(),
             self.path[1].to_drawing_point(),
             self.path[2].to_drawing_point()]
    }

    fn is_valid(&self) -> bool {
        let p0 = self.path[0];
        let p1 = self.path[1];
        let p2 = self.path[2];

        if p0 == p1 || p0 == p2 || p1 == p2 {
            false
        } else {
            p0.angle(p1, p2) > MINIMUM_DEGREES &&
                p1.angle(p2, p0) > MINIMUM_DEGREES &&
                p2.angle(p0, p1) > MINIMUM_DEGREES
        }
    }

    fn new(vertices: Vec<PrimitivePoint>) -> Box<Triangle> {
        if vertices.len() != 3 {
            panic!("Triangles have 3 vertices, not {}!", vertices.len());
        } else {
            Box::new(Triangle {color: Rgba([0, 0, 0, 128]), path: [vertices[0], vertices[1], vertices[2]]})
        }
    }
}

impl RandomPolygon for Triangle {
    fn random(width: u32, height: u32, border_extension: i32, seed: u64) -> Box<Polygon> {
        let p0 = PrimitivePoint::random_point(width, height, seed);
        let p1 = p0.random_point_in_radius(border_extension, seed);
        let p2 = p0.random_point_in_radius(border_extension, seed);

        let mut tri = Triangle::new(vec![p0, p1, p2]);
        tri.mutate(width, height, seed);

        tri
    }
}

impl Polygon for Triangle {
    fn mutate(&mut self, width: u32, height: u32, seed: u64) {
        let mut rng = if seed != 0 {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::seed_from_u64(get_time_seed())
        };

        let mut i = 0;
        loop {
            i += 1;
            let r = rng.gen_range(0, 3);

            self.path[r].mutate(width, height, seed);

            if self.is_valid() {
                break;
            }
            if i > 10000 {
                panic!("Too many mutation loops!");
            }
        }
    }

    fn contains_pixel(&self, x: i32, y: i32) -> bool {
        let p = PrimitivePoint::new(x, y);

        let w0 = orient_2d(self.path[1], self.path[2], p);
        let w1 = orient_2d(self.path[2], self.path[0], p);
        let w2 = orient_2d(self.path[0], self.path[1], p);
        w0 >= 0 && w1 >= 0 && w2 >= 0
    }

    fn bounding_box(&self) -> [PrimitivePoint; 2] {
        [PrimitivePoint::new(min(self.path[0].x, min(self.path[1].x, self.path[2].x)),
                             min(self.path[0].y, min(self.path[1].y, self.path[2].y))),

         PrimitivePoint::new(max(self.path[0].x, max(self.path[1].x, self.path[2].x)),
                             max(self.path[0].y, max(self.path[1].y, self.path[2].y)))]
    }

    fn as_svg(&self, scale: f64) -> String {
        format!("<polygon fill=\"#{:X}{:X}{:X}\" fill-opacity=\"{:.5}\" points=\"{},{} {},{} {},{}\" />",
                self.color.data[0], self.color.data[1], self.color.data[2],
                self.color.data[3] as f64 / 255.0,
                (self.path[0].x as f64 * scale) as i32, (self.path[0].y as f64 * scale) as i32,
                (self.path[1].x as f64 * scale) as i32, (self.path[1].y as f64 * scale) as i32,
                (self.path[2].x as f64 * scale) as i32, (self.path[2].y as f64 * scale) as i32)
    }

    fn paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();

        let mut tri_image: ImageBuffer<Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_pixel(width as u32, height as u32, image::Rgba([0, 0, 0, 0]));
        let mut output = image.clone();

        tri_image = draw_convex_polygon(&tri_image, &(self.get_drawing_points()), self.color);

        overlay(&mut output, &tri_image, 0, 0);

        output
    }

    fn scaled_paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>, scale: f64) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();

        let mut tri_image: ImageBuffer<Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_pixel(width as u32, height as u32, image::Rgba([0, 0, 0, 0]));
        let mut output = image.clone();

        let mut p0 = self.path[0].clone();
        let mut p1 = self.path[1].clone();
        let mut p2 = self.path[2].clone();

        p0.x = (p0.x as f64 * scale) as i32;
        p0.y = (p0.y as f64 * scale) as i32;
        p1.x = (p1.x as f64 * scale) as i32;
        p1.y = (p1.y as f64 * scale) as i32;
        p2.x = (p2.x as f64 * scale) as i32;
        p2.y = (p2.y as f64 * scale) as i32;

        let dp0 = p0.to_drawing_point();
        let dp1 = p1.to_drawing_point();
        let dp2 = p2.to_drawing_point();


        tri_image = draw_convex_polygon(&tri_image, &vec![dp0, dp1, dp2], self.color);

        overlay(&mut output, &tri_image, 0, 0);

        output
    }

    fn set_color_using(&mut self, image: &PrimitiveImage) {
        self.color = image.target_average_color_in_polygon(&Box::new(*self));
    }
}

///
/// https://fgiesen.wordpress.com/2013/02/08/triangle-rasterization-in-practice/
/// Compute the determinant |p0.x p1.x p2.x|
///                         |p0.y p1.y p2.y|
///                         | 1    1    1  |
/// If this is positive
///
fn orient_2d(p0: PrimitivePoint, p1: PrimitivePoint, p2: PrimitivePoint) -> i32 {
    (p1.x-p0.x)*(p2.y-p0.y) - (p1.y-p0.y)*(p2.x-p0.x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orient_2d() {
        let mut p1 = PrimitivePoint::new(1, 1);
        let mut p2 = PrimitivePoint::new(1, 1);
        let mut p3 = PrimitivePoint::new(1, 1);

        // |1 1 1|
        // |1 1 1| = 0
        // |1 1 1|
        let mut expected = 0;
        assert_eq!(orient_2d(p1, p2, p3), expected);

        p1.x = 1;
        p1.y = 0;
        p2.x = 0;
        p2.y = 1;
        p3.x = 1;
        p3.y = 1;

        // |1 0 1|
        // |0 1 1| = -1
        // |1 1 1|
        expected = -1;
        assert_eq!(orient_2d(p1, p2, p3), expected);

        p3.x = 0;
        p3.y = 0;

        // |1 0 0|
        // |0 1 0| = 1
        // |1 1 1|
        expected = 1;
        assert_eq!(orient_2d(p1, p2, p3), expected);
    }
}