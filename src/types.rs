pub struct WorldX(f64);

impl From<f64> for WorldX {
    fn from(x: f64) -> Self {
        Self(x)
    }
}

pub struct WorldY(f64);

impl From<f64> for WorldY {
    fn from(y: f64) -> Self {
        Self(y)
    }
}

pub struct WorldPosition {
    x: WorldX,
    y: WorldY,
}

impl From<(WorldX, WorldY)> for WorldPosition {
    fn from(p: (WorldX, WorldY)) -> Self {
        Self { x: p.0, y: p.1 }
    }
}

pub struct EntityIndex(usize);

impl From<usize> for EntityIndex {
    fn from(i: usize) -> Self {
        Self(i)
    }
}
