extern crate primitive_image;
extern crate imageproc;
extern crate image;
#[macro_use] extern crate log;
extern crate simplelog;

mod arguments;

use structopt::StructOpt;
use primitive_image::primitive_image::PrimitiveImage;
use simplelog::*;
use primitive_image::shape::ShapeType;
use primitive_image::utilities::get_rng;
use rand::Rng;
use image::Rgba;
use std::i64;

fn run(image: &mut PrimitiveImage, number_of_shapes: u32, max_age: u32, seed: u64, s: String) {
    let mut n = 0;

    let mut rng = get_rng(seed);

    while n < number_of_shapes {

        let shape = match s.as_ref() {
            "TRIANGLE" => ShapeType::Triangle,
            "QUADRATIC" => ShapeType::QuadraticBezier,
            "CUBIC" => ShapeType::CubicBezier,
            "RECTANGLE" => ShapeType::Rectangle,
            "ELLIPSE" => ShapeType::Ellipse,
            "MIXED" => {
                let r = rng.gen_range(0, 5);
                match r {
                    0 => ShapeType::Triangle,
                    1 => ShapeType::QuadraticBezier,
                    2 => ShapeType::CubicBezier,
                    3 => ShapeType::Ellipse,
                    4 => ShapeType::Rectangle
                }
            },
            _ => panic!("Unsupported shape: {}", s)
        };

        if image.add_new_shape(max_age, &shape, seed) {
            n += 1;
            info!("Added {:?} (Shape #{})", &shape, n);
        } else {
            trace!("Failed to add shape (#{})", (n+1));
        }
    }
}

fn main() {
    let opt = arguments::Opt::from_args();

    let log_level = match opt.v {
        0 => LevelFilter::Off,
        1 => LevelFilter::Info,
        _ => LevelFilter::Trace
    };

    SimpleLogger::init(log_level, Config::default()).unwrap();

    let input_path = opt.in_path;

    // Parse background color if provided
    let background = if opt.background_color.is_some() {
        let background_color: String = opt.background_color.unwrap();

        if background_color.len() != 6 {
            panic!("Incorrect background color format: {}", background_color);
        }

        let mut data: [u8; 4] = [0, 0, 0, 0];

        data[0] = i64::from_str_radix(&background_color[0..2], 16).ok().unwrap() as u8;
        data[1] = i64::from_str_radix(&background_color[2..4], 16).ok().unwrap() as u8;
        data[2] = i64::from_str_radix(&background_color[4..6], 16).ok().unwrap() as u8;

        Some(Rgba(data))
    } else {
        None
    };

    let mut image = PrimitiveImage::from_path(input_path, opt.scale_to, background);

    run(&mut image, opt.n, opt.max_age, opt.seed, opt.shape);

    image.save_to(opt.out_path);
}