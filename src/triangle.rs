use crate::shape::{Shape, RandomShape};
use crate::point::PrimitivePoint;
use crate::primitive_image::PrimitiveImage;
use image::Rgba;
use imageproc::drawing::Point;
use std::cmp::{min, max};
use rand;
use rand::Rng;
use image::ImageBuffer;
use imageproc::drawing::draw_convex_polygon;
use image::imageops::overlay;
use crate::utilities::get_rng;

const MINIMUM_DEGREES: f64 = 15.0;
const MAXIMUM_MUTATION_ATTEMPTS: u32 = 100_000;

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

    ///
    /// Determine if this triangle is valid
    ///
    /// A triangle is valid if none of is points are equal to each other and
    /// if all of its angles are at least `MINIMUM_DEGREES` in magnitude
    ///
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



    ///
    /// Determine if this triangle contains the point (`x`, `y`)
    ///
    fn contains_pixel(&self, x: i32, y: i32) -> bool {
        let p = PrimitivePoint::new(x, y);

        let w0 = orient_2d(self.path[1], self.path[2], p);
        let w1 = orient_2d(self.path[2], self.path[0], p);
        let w2 = orient_2d(self.path[0], self.path[1], p);
        w0 >= 0 && w1 >= 0 && w2 >= 0
    }
}

impl RandomShape for Triangle {

    ///
    /// Generate a random Triangle within the bounds given
    /// `border_extension` is the maximum distance outside of the border a triangle is allowed to go
    ///     It must be >= 1
    ///
    fn random(width: u32, height: u32, border_extension: i32, seed: u64) -> Box<Shape> {
        let p0 = PrimitivePoint::random_point(width, height, seed);
        let p1 = p0.random_point_in_radius(border_extension, seed);
        let p2 = p0.random_point_in_radius(border_extension, seed);

        let mut tri = Triangle::new(vec![p0, p1, p2]);
        tri.mutate(width, height, seed);

        tri
    }
}

impl Shape for Triangle {

    ///
    /// Attempt to mutate this triangle
    /// Guarantees that the triangle remains valid
    /// Does not recolor the triangle
    ///
    fn mutate(&mut self, width: u32, height: u32, seed: u64) {
        let mut rng = get_rng(seed);

        let mut i = 0;
        loop {
            i += 1;
            let r = rng.gen_range(0, 3);

            self.path[r].mutate(width, height, seed);

            if self.is_valid() {
                break;
            }
            if i > MAXIMUM_MUTATION_ATTEMPTS {
                panic!("Triangle: Too many mutation loops!");
            }
        }
    }

    fn get_pixels(&self) -> Vec<PrimitivePoint> {
        let bounding_box = self.bounding_box();

        let min_x = bounding_box[0].x;
        let min_y = bounding_box[0].y;
        let max_x = bounding_box[1].x;
        let max_y = bounding_box[1].y;

        let mut pixels = vec![];

        for x in min_x..max_x {
            for y in min_y..max_y {
                if self.contains_pixel(x as i32, y as i32) {
                    pixels.push(PrimitivePoint::new(x, y));
                }
            }
        }

        pixels
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

    // Suppress intellij inspection for E0308 (false positive)
    //noinspection RsTypeCheck
    fn paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();

        let mut tri_image: ImageBuffer<Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_pixel(width as u32, height as u32, image::Rgba([0, 0, 0, 0]));
        let mut output = image.clone();

        tri_image = draw_convex_polygon(&tri_image, &(self.get_drawing_points()), self.color);

        overlay(&mut output, &tri_image, 0, 0);

        output
    }

    // Suppress intellij inspection for E0308 (false positive)
    //noinspection RsTypeCheck
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
        self.color = image.target_average_color_in_shape(&Box::new(*self));
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

    #[test]
    fn test_is_valid() {
        // A "triangle" with points all identical - not valid
        let p1 = PrimitivePoint::new(0, 0);
        let p2 = PrimitivePoint::new(0, 0);
        let p3 = PrimitivePoint::new(0, 0);
        let tri = Triangle{path: [p1, p2, p3], color: Rgba([0, 0, 0, 0])};
        assert_eq!(tri.is_valid(), false);

        // A "triangle" with two identical points - not valid
        let p1 = PrimitivePoint::new(5, 0);
        let p2 = PrimitivePoint::new(0, 0);
        let p3 = PrimitivePoint::new(0, 0);
        let tri = Triangle{path: [p1, p2, p3], color: Rgba([0, 0, 0, 0])};
        assert_eq!(tri.is_valid(), false);

        // A "triangle" with two identical points - not valid
        let p1 = PrimitivePoint::new(0, 0);
        let p2 = PrimitivePoint::new(5, 0);
        let p3 = PrimitivePoint::new(0, 0);
        let tri = Triangle{path: [p1, p2, p3], color: Rgba([0, 0, 0, 0])};
        assert_eq!(tri.is_valid(), false);

        // A "triangle" with two identical points - not valid
        let p1 = PrimitivePoint::new(0, 0);
        let p2 = PrimitivePoint::new(0, 0);
        let p3 = PrimitivePoint::new(5, 0);
        let tri = Triangle{path: [p1, p2, p3], color: Rgba([0, 0, 0, 0])};
        assert_eq!(tri.is_valid(), false);

        // A right triangle with vertices at the origin and on the x-axis - valid
        let p1 = PrimitivePoint::new(0, 0);
        let p2 = PrimitivePoint::new(5, 0);
        let p3 = PrimitivePoint::new(0, 5);
        let tri = Triangle{path: [p1, p2, p3], color: Rgba([0, 0, 0, 0])};
        assert_eq!(tri.is_valid(), true);

        // A right triangle with the smallest angle ~15.5 - valid
        let p1 = PrimitivePoint::new(0, 0);
        let p2 = PrimitivePoint::new(5, 0);
        let p3 = PrimitivePoint::new(0, 18);
        let tri = Triangle{path: [p1, p2, p3], color: Rgba([0, 0, 0, 0])};
        assert_eq!(tri.is_valid(), true);

        // A right triangle with the smallest angle ~14.7 - not valid
        let p1 = PrimitivePoint::new(0, 0);
        let p2 = PrimitivePoint::new(5, 0);
        let p3 = PrimitivePoint::new(0, 19);
        let tri = Triangle{path: [p1, p2, p3], color: Rgba([0, 0, 0, 0])};
        assert_eq!(tri.is_valid(), false);
    }
}