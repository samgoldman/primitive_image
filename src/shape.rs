use crate::point::PrimitivePoint;
use crate::primitive_image::PrimitiveImage;
use image::ImageBuffer;
use image::Rgba;
use std::fmt::Debug;

#[derive(Debug)]
pub enum ShapeType {
    Triangle,
    CubicBezier
}

pub trait Shape: ShapeClone + Debug {
    fn mutate(&mut self, width: u32, height: u32, seed: u64);
    fn contains_pixel(&self, x: i32, y: i32) -> bool;
    fn bounding_box(&self) -> [PrimitivePoint; 2];
    fn as_svg(&self, scale: f64) -> String;
    fn paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>>;
    fn scaled_paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>, scale: f64) -> ImageBuffer<Rgba<u8>, Vec<u8>>;
    fn set_color_using(&mut self, image: &PrimitiveImage);
}

pub trait ShapeClone {
    fn clone_box(&self) -> Box<Shape>;
}

impl<T> ShapeClone for T
where
    T: 'static + Shape + Clone,
{
    fn clone_box(&self) -> Box<Shape> {
        Box::new(self.clone())
    }
}

impl Clone for Box<Shape> {
    fn clone(&self) -> Box<Shape> {
        self.clone_box()
    }
}

pub trait RandomShape {
    fn random(width: u32, height: u32, border_extension: i32, seed: u64) -> Box<Shape>;
}