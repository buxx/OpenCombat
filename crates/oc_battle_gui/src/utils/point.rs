#[macro_export]
macro_rules! cursor_to {
    ($point:expr, $camera:expr, $w:expr, WorldVec2) => {{
        let (camera, transform) = *$camera;
        let point = camera.viewport_to_world_2d(transform, $point);
        oc_utils::let_ok!(point = point, return);
        let point = oc_root::geo::ScreenVec2::new(point.x, point.y);
        WorldVec2::from_(point, &$w)
    }};
}
