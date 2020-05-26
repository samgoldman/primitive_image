extern crate primitive_image;
extern crate imageproc;
extern crate image;
extern crate simplelog;

mod arguments;

use structopt::StructOpt;
use simplelog::*;
use primitive_image::runner::run;
use std::i64;
use primitive_image::primitive_image::PrimitiveImage;
use image::Rgba;

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