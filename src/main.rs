extern crate primitive_image;
extern crate imageproc;
extern crate image;
#[macro_use] extern crate log;
extern crate simplelog;

mod arguments;

use structopt::StructOpt;
use primitive_image::primitive_image::PrimitiveImage;
use simplelog::*;
use primitive_image::polygon::PolygonType::triangle;

fn run(image: &mut PrimitiveImage, number_of_shapes: u32, max_age: u32, seed: u64) {
    let mut n = 0;

    while n < number_of_shapes {
        if image.add_new_shape(max_age, triangle, seed) {
            n += 1;
            info!("Added shape #{}", n);
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

    let mut image = PrimitiveImage::from_path(input_path, opt.scale_to, None);

    run(&mut image, opt.n, opt.max_age, opt.seed);

    image.save_to(opt.out_path);
}