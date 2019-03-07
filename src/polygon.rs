use crate::point::PrimitivePoint;
use crate::primitive_image::PrimitiveImage;
use image::ImageBuffer;
use image::Rgba;
use std::fmt::Debug;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum PolygonType {
    triangle
}

pub trait Polygon: PolygonClone + Debug {
    fn mutate(&mut self, width: u32, height: u32, seed: u64);
    fn contains_pixel(&self, x: i32, y: i32) -> bool;
    fn bounding_box(&self) -> [PrimitivePoint; 2];
    fn as_svg(&self, scale: f64) -> String;
    fn paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>>;
    fn scaled_paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>, scale: f64) -> ImageBuffer<Rgba<u8>, Vec<u8>>;
    fn set_color_using(&mut self, image: &PrimitiveImage);
}

pub trait PolygonClone {
    fn clone_box(&self) -> Box<Polygon>;
}

impl<T> PolygonClone for T
where
    T: 'static + Polygon + Clone,
{
    fn clone_box(&self) -> Box<Polygon> {
        Box::new(self.clone())
    }
}

impl Clone for Box<Polygon> {
    fn clone(&self) -> Box<Polygon> {
        self.clone_box()
    }
}

pub trait RandomPolygon {
    fn random(width: u32, height: u32, border_extension: i32, seed: u64) -> Box<Polygon>;
}