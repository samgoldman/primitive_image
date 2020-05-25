use rand_distr::Normal;
use imageproc::drawing::Point;
use super::utilities::*;
use rand::Rng;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PrimitivePoint {
    pub x: i32,
    pub y: i32
}
impl Eq for PrimitivePoint {}
impl PrimitivePoint {
    pub fn new(x: i32, y: i32) -> PrimitivePoint {
        PrimitivePoint {x, y}
    }

    ///
    /// Mutate the point's x and y coordinates
    /// Keeps the point within 5 pixels outside of the standard image border
    /// Uses a standard deviation of 16, with a mean of 0, for the mutation
    ///
    pub fn mutate(&mut self, width: u32, height: u32, seed: u64) {
        let mut rng = get_rng(seed);

        let border_extension = 5;

        let normal = Normal::new(0.0, 16.0).unwrap();

        self.x = clamp(self.x + (rng.sample(normal) as i32), -1 * border_extension, width as i32 + border_extension);
        self.y = clamp(self.y + (rng.sample(normal) as i32), -1 * border_extension, height as i32 + border_extension);
    }

    ///
    /// Convert this point to the `Point` format used by imageproc
    ///
    pub fn to_drawing_point(&self) -> Point<i32> {
        Point::new(self.x, self.y)
    }

    /// Returns the value in degrees of the angle formed between this point and p2 and p3
    pub fn angle(&self, p2: PrimitivePoint, p3: PrimitivePoint) -> f64 {
        let dx1 = (p2.x - self.x) as f64;
        let dy1 = (p2.y - self.y) as f64;
        let dx2 = (p3.x - self.x) as f64;
        let dy2 = (p3.y - self.y) as f64;

        let d1 = sqrt(dx1*dx1 + dy1*dy1);
        let d2 = sqrt(dx2*dx2 + dy2*dy2);

        let rdx1 = dx1/d1;
        let rdy1 = dy1/d1;
        let rdx2 = dx2/d2;
        let rdy2 = dy2/d2;

        degrees(acos(rdx1*rdx2 + rdy1*rdy2))
    }

    ///
    /// Return a new Primitive point within the rectangular bounds provided
    ///
    pub fn random_point(width: u32, height: u32, seed: u64) -> PrimitivePoint {
        let mut rng = get_rng(seed);

        let rand_x = rng.gen_range(0, width as i32);
        let rand_y = rng.gen_range(0, height as i32);

        PrimitivePoint::new(rand_x, rand_y)
    }

    ///
    /// Return a new PrimitivePoint with `radius` pixels of this point
    ///
    pub fn random_point_in_radius(&self, radius: i32, seed: u64) -> PrimitivePoint {
        let mut rng = get_rng(seed);

        PrimitivePoint::new(rng.gen_range(self.x - radius, self.x + radius),
                            rng.gen_range(self.y - radius, self.y + radius))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_point() {
        let seed: u64 = 42;
        assert_eq!(PrimitivePoint::random_point(10, 10, seed), PrimitivePoint::new(8, 5));
    }

    #[test]
    fn test_point_in_radius() {
        let seed: u64 = 42;
        let p = PrimitivePoint::new(5, 5);
        // Like in `test_random_point`, both ranges should be [0-10), so the output point should be the same, given the same seed
        assert_eq!(p.random_point_in_radius(5, seed), PrimitivePoint::new(8, 5));
    }

    #[test]
    fn test_angle() {
        // A right triangle with the smallest angle ~15.5
        let p1 = PrimitivePoint::new(0, 0);
        let p2 = PrimitivePoint::new(5, 0);
        let p3 = PrimitivePoint::new(0, 18);
        assert_eq!(p3.angle(p1, p2) as u32, 15);

        // A right triangle with the smallest angle ~14.7
        let p1 = PrimitivePoint::new(0, 0);
        let p2 = PrimitivePoint::new(5, 0);
        let p3 = PrimitivePoint::new(0, 19);
        assert_eq!(p3.angle(p1, p2) as u32, 14);

        // A right triangle
        let p1 = PrimitivePoint::new(0, 0);
        let p2 = PrimitivePoint::new(5, 0);
        let p3 = PrimitivePoint::new(0, 5);
        assert_eq!(p1.angle(p2, p3) as u32, 90);

        // A right triangle with 45-45
        let p1 = PrimitivePoint::new(0, 0);
        let p2 = PrimitivePoint::new(5, 0);
        let p3 = PrimitivePoint::new(0, 5);
        assert_eq!(p2.angle(p1, p3) as u32, 45);

    }

    #[test]
    fn test_mutate() {
        let seed = 42;

        let mut p = PrimitivePoint::new(0, 0);
        p.mutate(10, 10, seed);
        assert_eq!(p.x, 0); // Based on prior executions
        assert_eq!(p.y, -3);
    }
}