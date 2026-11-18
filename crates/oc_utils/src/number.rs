pub fn almost_equal(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}
