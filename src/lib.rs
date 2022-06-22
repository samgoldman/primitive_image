#![feature(explicit_generic_args_with_impl_trait)]

extern crate image;
extern crate imageproc;
extern crate rand;
#[macro_use]
extern crate log;

pub mod cubic_bezier;
pub mod ellipse;
pub mod point;
pub mod primitive_image;
pub mod quadratic_bezier;
pub mod rectangle;
pub mod runner;
pub mod shape;
pub mod triangle;
pub mod utilities;
