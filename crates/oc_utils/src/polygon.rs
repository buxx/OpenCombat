use oc_root::geo::WorldVec2;

use crate::d2::Shape;

pub trait Polygon {
    fn points(&self) -> &[WorldVec2];

    // Warning: IA generated function
    fn contains(&self, p: WorldVec2) -> bool {
        let points = self.points();

        let Some(mut j) = points.len().checked_sub(1) else {
            return false;
        };

        let mut result = false;
        for i in 0..points.len() {
            let (xi, yi) = (points[i].x, points[i].y);
            let (xj, yj) = (points[j].x, points[j].y);

            if (yi > p.y) != (yj > p.y) && p.x < (xj - xi) * (p.y - yi) / (yj - yi) + xi {
                result = !result;
            }

            j = i;
        }

        result
    }

    // Warning: IA generated function
    fn include_shape(&self, shape: &Shape) -> bool {
        let corners = [
            (shape.top_left.x, shape.top_left.y),
            (shape.top_right.x, shape.top_right.y),
            (shape.bottom_right.x, shape.bottom_right.y),
            (shape.bottom_left.x, shape.bottom_left.y),
        ];

        // 1. Every corner must be inside.
        if !corners
            .iter()
            .all(|&(x, y)| self.contains(WorldVec2::new(x, y)))
        {
            return false;
        }

        // 2. No polygon edge may cross a shape edge (handles concave polygons).
        let points = self.points();
        let n = points.len();
        for i in 0..n {
            let a = (points[i].x, points[i].y);
            let b = (points[(i + 1) % n].x, points[(i + 1) % n].y);

            for k in 0..4 {
                let c = corners[k];
                let d = corners[(k + 1) % 4];
                if segments_cross(a, b, c, d) {
                    return false;
                }
            }
        }

        true
    }
}

type P = (f32, f32);

// Warning: IA generated
#[inline]
fn orient(a: P, b: P, c: P) -> f32 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

// Warning: IA generated function
/// True only for a proper crossing (segments intersect at a single interior
/// point of both). Touching or collinear overlap is not considered a crossing.
fn segments_cross(a: P, b: P, c: P, d: P) -> bool {
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);

    (o1 > 0.0) != (o2 > 0.0)
        && o1 != 0.0
        && o2 != 0.0
        && (o3 > 0.0) != (o4 > 0.0)
        && o3 != 0.0
        && o4 != 0.0
}
