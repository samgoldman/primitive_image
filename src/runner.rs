use crate::cubic_bezier::CubicBezier;
use crate::primitive_image::PrimitiveImage;
use crate::quadratic_bezier::QuadraticBezier;
use crate::rectangle::Rectangle;
use crate::triangle::Triangle;
use crate::ellipse::Ellipse;
use crate::utilities::get_rng;
use rand::Rng;

pub fn run(image: &mut PrimitiveImage, number_of_shapes: u32, max_age: u32, seed: u64, s: String) {
    let mut n = 0;

    let mut rng = get_rng(seed);

    while n < number_of_shapes {
        let res = match s.as_ref() {
            "TRIANGLE" => image.add_new_shape::<Triangle>(max_age, &mut rng),
            "QUADRATIC" => image.add_new_shape::<QuadraticBezier>(max_age, &mut rng),
            "CUBIC" => image.add_new_shape::<CubicBezier>(max_age, &mut rng),
            "RECTANGLE" => image.add_new_shape::<Rectangle>(max_age, &mut rng),
            "ELLIPSE" => image.add_new_shape::<Ellipse>(max_age, &mut rng),
            "MIXED" => {
                let r = rng.gen_range(0..5);
                match r {
                    0 => image.add_new_shape::<Triangle>(max_age, &mut rng),
                    1 => image.add_new_shape::<QuadraticBezier>(max_age, &mut rng),
                    2 => image.add_new_shape::<CubicBezier>(max_age, &mut rng),
                    3 => image.add_new_shape::<Rectangle>(max_age, &mut rng),
                    4 => image.add_new_shape::<Ellipse>(max_age, &mut rng),
                    _ => panic!("This should never be reached if the range is set properly!"),
                }
            }
            _ => panic!("Unsupported shape: {}", s),
        };

        if res {
            n += 1;
            info!("Added #{}", n);
        } else {
            trace!("Failed to add shape (#{})", (n + 1));
        }
    }
}
