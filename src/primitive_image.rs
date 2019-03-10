use crate::triangle::Triangle;
use crate::cubic_bezier::CubicBezier;
use crate::quadratic_bezier::QuadraticBezier;
use crate::rectangle::Rectangle;
use crate::ellipse::Ellipse;
use crate::shape::{Shape, RandomShape};

use image::{open, Rgba, ImageBuffer};
use image::imageops::{resize, Nearest};
use std::cmp::max;
use std::option::Option;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::error::Error;
use std::io::Write;
use imageproc::stats::{root_mean_squared_error};
use crate::shape::ShapeType;
use crate::utilities::rgb_to_hex;

const BORDER_EXTENSION: i32 = 6;

#[derive(Clone)]
pub struct PrimitiveImage {
    target: ImageBuffer<Rgba<u8>, Vec<u8>>,
    approximation: ImageBuffer<Rgba<u8>, Vec<u8>>,
    scale: f64,
    shapes: Vec<Box<Shape>>,
    background: Rgba<u8>
}
impl PrimitiveImage {
    pub fn from_path(path: PathBuf, scale_to: u32, background: Option<Rgba<u8>>) -> PrimitiveImage {
        let original = open(&path)
            .expect(&format!("Could not load image at {:?}", path))
            .to_rgba();

        let (original_width, original_height) = original.dimensions();

        let background = if background.is_some() {
            background.unwrap()
        } else {
            average_color(&original)
        };

        // Set the scale so that when the image is resized, the largest
        // dimension is now scale_to pixels in length
        // If scale is <= 0, use the original image size
        let scale = if scale_to > 0 {
            scale_to as f64 / max(original_width , original_height) as f64
        } else {
            1.0
        };

        let new_width = (original_width as f64 * scale) as u32;
        let new_height = (original_height as f64 * scale) as u32;

        let resized = resize(&original, new_width, new_height, Nearest);

        let approximation = ImageBuffer::from_pixel(new_width, new_height, background);

        PrimitiveImage {target: resized, approximation, scale, background, shapes: vec![]}
    }

    pub fn target_average_color_in_shape(&self, shape: &Box<impl Shape>) -> Rgba<u8> {
        average_color_in_shape(&self.target, shape)
    }

    pub fn save_to(&self, path: PathBuf) {
        let extension = path.extension();

        match extension {
            None => panic!("Can't save to file {:?} (no extension found!)", path),
            Some(os_str) => {
                match os_str.to_str() {
                    Some("svg") => {self.save_to_svg(path)},
                    Some("png") | Some("jpg") | Some("bmp") | Some("ico") | Some("gif") => self.save_to_img(path),
                    _ => error!("Invalid save file type: {:?}", extension)
                }
            }
        }
    }

    fn as_svg(&self) -> String {
        let mut result = String::new();

        let (scaled_width, scaled_height) = self.target.dimensions();
        let inverted_scale = 1.0 / self.scale;
        let original_width = (scaled_width as f64 * inverted_scale) as u32;
        let original_height = (scaled_height as f64 * inverted_scale) as u32;

        result += &format!("<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{}\" height=\"{}\">",
                 original_width, original_height);

        // Use an SVG transform to resize all of the polygons (it's easier to have someone else do the math)
        //result += &format!("<g transform=\"scale({})\">", inverted_scale);

        // Add the background
        result += &format!("<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\" />",
                 original_width, original_height,
                 rgb_to_hex(self.background));

        result += &format!("<g>");


        // Add the polygons!
        for polygon in self.shapes.iter() {
            result += &format!("{}", polygon.as_svg(inverted_scale));
        }

        //result += &format!("</g></g></svg>");
        result += &format!("</g></svg>");

        result
    }

    pub fn save_to_svg(&self, path: PathBuf) {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(path);

        if file.is_err()  {
            let err = file.unwrap_err();
            panic!("{}", err.description());
        }

        let mut svg = file.unwrap();
        write!(&svg, "{}", self.as_svg()).unwrap();
        svg.flush().unwrap();
    }

    /// Save the current approximation in an image format
    ///
    ///  # Arguments
    ///
    /// * `path` - The path to save the image to. It must be a format supported by the image library
    ///
    pub fn save_to_img(&self, path: PathBuf) {
        let (scaled_width, scaled_height) = self.target.dimensions();
        let inverted_scale = 1.0 / self.scale;
        let original_width = (scaled_width as f64 * inverted_scale) as u32;
        let original_height = (scaled_height as f64 * inverted_scale) as u32;

        let mut img = ImageBuffer::from_pixel(original_width, original_height, self.background);

        for poly in self.shapes.iter() {
            img = poly.scaled_paint_on(&img, inverted_scale);
        }

        img.save(path).unwrap();
    }

    /// Returns the current approximation's score.
    ///
    /// Uses imageproc's Root Mean Squared Error function on the target and approximation images
    pub fn score(&self) -> f64 {
        root_mean_squared_error(&self.target, &self.approximation)
    }

    pub fn add_new_shape(&mut self, max_age: u32, shape_type: &ShapeType, seed: u64) -> bool {
        // Initialize a random shape and give it a color
        let mut shape = match shape_type {
                ShapeType::Triangle => Triangle::random(self.width(), self.height(), BORDER_EXTENSION, seed),
                ShapeType::CubicBezier => CubicBezier::random(self.width(), self.height(), BORDER_EXTENSION, seed),
                ShapeType::QuadraticBezier => QuadraticBezier::random(self.width(), self.height(), BORDER_EXTENSION, seed),
                ShapeType::Rectangle => Rectangle::random(self.width(), self.height(), BORDER_EXTENSION, seed),
                ShapeType::Ellipse => Ellipse::random(self.width(), self.height(), BORDER_EXTENSION, seed)
        };
        shape.set_color_using(self);

        // The initial triangle is the best so far
        let mut best_shape = shape.clone();
        let mut best_image = self.clone();
        best_image.approximation = best_shape.paint_on(&best_image.approximation);
        let mut best_score = best_image.score();


        let mut age = 0;
        // Loop until max_age mutations fail to yield and improvement
        while age < max_age {
            // Mutate the shape and update its color
            shape.mutate(self.width(), self.height(), seed);
            shape.set_color_using(self);

            // Determine its score
            let mut new_image = self.clone();
            new_image.approximation = shape.paint_on(&new_image.approximation);
            let new_score = new_image.score();

            // Trying to minimize score (smaller score = closer approximation to the target)
            if new_score < best_score {
                best_score = new_score;
                best_shape = shape.clone();

                // Reset age if an improvement was made
                age = 0;
            } else {
                // Reset the shape and increment age
                shape = best_shape.clone();
                age += 1;
            }

            trace!("Age: {}, best score: {}", age, best_score);
        }

        // Only add the shape if it is an improvement over the current approximation
        // Return true if a shape was added
        if best_score < self.score() {
            trace!("Adding shape {:?}", best_shape);
            self.approximation = best_shape.paint_on(&self.approximation);
            self.shapes.push(best_shape);
            true
        } else {
            false
        }
    }

    fn width(&self) -> u32 {
        self.target.dimensions().0
    }

    fn height(&self) -> u32 {
        self.target.dimensions().1
    }
}

fn average_color_in_shape(image: &ImageBuffer<Rgba<u8>, Vec<u8>>, shape: &Box<impl Shape>) -> Rgba<u8> {
    let (width, height) = image.dimensions();

    let pixels = shape.get_pixels();

    let mut channel_sums: [i64; 4] = [0, 0, 0, 0];

    let mut num_pixels: i64 = 0;

    for pixel in pixels {
        if pixel.x < 0 || pixel.x >= width as i32 || pixel.y < 0 || pixel.y >= height as i32 {
            continue;
        }

        num_pixels += 1;
        let image_pixel = image.get_pixel(pixel.x as u32, pixel.y as u32);

        channel_sums[0] += image_pixel[0] as i64;
        channel_sums[1] += image_pixel[1] as i64;
        channel_sums[2] += image_pixel[2] as i64;
    }

    let mut average_pixels: [u8; 4] = [0, 0, 0, 0];

    if num_pixels > 0 {
        average_pixels[0] = (channel_sums[0] / num_pixels) as u8;
        average_pixels[1] = (channel_sums[1] / num_pixels) as u8;
        average_pixels[2] = (channel_sums[2] / num_pixels) as u8;
        average_pixels[3] = 128;
    }

    Rgba(average_pixels)
}

fn average_color(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Rgba<u8> {
    let mut channel_sums: [i64; 4] = [0, 0, 0, 0];

    let mut num_pixels: i64 = 0;

    for pixel in image.pixels() {
        num_pixels += 1;

        channel_sums[0] += pixel[0] as i64;
        channel_sums[1] += pixel[1] as i64;
        channel_sums[2] += pixel[2] as i64;
    }

    let mut average_pixels: [u8; 4] = [0, 0, 0, 0];

    if num_pixels > 0 {
        average_pixels[0] = (channel_sums[0] / num_pixels) as u8;
        average_pixels[1] = (channel_sums[1] / num_pixels) as u8;
        average_pixels[2] = (channel_sums[2] / num_pixels) as u8;
        average_pixels[3] = 128;
    }

    Rgba(average_pixels)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utilities::sqrt;

    #[test]
    fn test_average_color() {
        // A the average of a solid colored image should be that color
        let test_im_1 = ImageBuffer::from_pixel(10, 10, Rgba([255, 128, 0, 128]));
        assert_eq!(average_color(&test_im_1), Rgba([255, 128, 0, 128]));

        // Average color should always return a color with alpha=128
        let test_im_2 = ImageBuffer::from_pixel(10, 10, Rgba([255, 128, 0, 255]));
        assert_eq!(average_color(&test_im_2), Rgba([255, 128, 0, 128]));

        // Test with two pixels, each a different color
        let mut test_im_3 = ImageBuffer::from_pixel(1, 2, Rgba([0, 0, 0, 128]));
        test_im_3.get_pixel_mut(0, 1).data = [10, 10, 10, 128];
        assert_eq!(average_color(&test_im_3), Rgba([5, 5, 5, 128]));
    }

    #[test]
    fn test_score() {
        let approximation = ImageBuffer::from_pixel(2, 2, Rgba([0, 0, 0, 128]));
        let target = ImageBuffer::from_pixel(2, 2, Rgba([10, 10, 10, 128]));
        let primitive = PrimitiveImage{target, approximation, background: Rgba([0, 0, 0, 0]), scale: 1.0, shapes: vec![]};

        // sqrt((Error(10.0)*Error(10.0)*NumChannelsWithError(3.0)*NumPixels(4.0))/(NumChannels(4.0)*NumPixels(4.0))
        let expected_score = sqrt((10.0*10.0*3.0*4.0)/(4.0*4.0));

        assert_eq!(primitive.score(), expected_score);
    }
}