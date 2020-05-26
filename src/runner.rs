use crate::primitive_image::PrimitiveImage;
use crate::utilities::get_rng;
use crate::shape::ShapeType;
use rand::Rng;

pub fn run(image: &mut PrimitiveImage, number_of_shapes: u32, max_age: u32, seed: u64, s: String) {
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
                    4 => ShapeType::Rectangle,
                    _ => panic!("This should never be reached if the range is set properly!")
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