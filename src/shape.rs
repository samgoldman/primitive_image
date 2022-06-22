use crate::point::PrimitivePoint;
use crate::primitive_image::PrimitiveImage;
use image::ImageBuffer;
use image::Rgba;
use rand::Rng;
use std::fmt::Debug;

#[derive(Debug)]
pub enum ShapeType {
    Triangle,
    CubicBezier,
    QuadraticBezier,
    Ellipse,
    Rectangle,
}

pub trait Shape: ShapeClone + Debug {
    fn mutate(&mut self, width: u32, height: u32, rng: &mut impl Rng) where Self: Sized;
    fn get_pixels(&self) -> Vec<PrimitivePoint>;
    fn as_svg(&self, scale: f64) -> String;
    fn paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>>;
    fn scaled_paint_on(
        &self,
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        scale: f64,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>>;
    fn set_color_using(&mut self, image: &PrimitiveImage);
}

pub trait ShapeClone {
    fn clone_box(&self) -> Box<dyn Shape>;
}

impl<T> ShapeClone for T
where
    T: 'static + Shape + Clone,
{
    fn clone_box(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Shape> {
    fn clone(&self) -> Box<dyn Shape> {
        self.clone_box()
    }
}

pub trait RandomShape {
    fn random(width: u32, height: u32, border_extension: i32, rng: &mut impl Rng) -> Self;
}
