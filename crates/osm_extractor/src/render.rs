//! Rasterize extracted layers with colors of the default OpenStreetMap theme (openstreetmap-carto).

use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, StrokeDash, Transform,
};

use crate::{
    feature::{DataType, Feature, FeatureCollection, Geometry, Point, Ring},
    geo::Projector,
};

/// openstreetmap-carto colors.
pub mod colors {
    pub const LAND: u32 = 0xf2efe9;
    pub const FOREST: u32 = 0xadd19e;
    pub const WATER: u32 = 0xaad3df;
    pub const BUILDING: u32 = 0xd9d0c9;
    pub const BUILDING_OUTLINE: u32 = 0xc4b6ab;
    pub const PEDESTRIAN_AREA: u32 = 0xdddde8;
}

fn color(rgb: u32) -> Color {
    Color::from_rgba8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255)
}

fn paint(rgb: u32) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color(color(rgb));
    paint.anti_alias = true;
    paint
}

pub struct Renderer {
    projector: Projector,
    pixels_per_meter: f64,
}

impl Renderer {
    pub fn new(projector: Projector, pixels_per_meter: f64) -> Self {
        Self {
            projector,
            pixels_per_meter,
        }
    }

    pub fn new_pixmap(&self, background: Option<u32>) -> Pixmap {
        let mut pixmap =
            Pixmap::new(self.projector.width, self.projector.height).expect("non zero size");
        if let Some(background) = background {
            pixmap.fill(color(background));
        }
        pixmap
    }

    /// Width in pixels of something `meters` wide, at least `min_px`.
    fn px(&self, meters: f64, min_px: f32) -> f32 {
        ((meters * self.pixels_per_meter) as f32).max(min_px)
    }

    pub fn draw(&self, pixmap: &mut Pixmap, type_: DataType, collection: &FeatureCollection) {
        match type_ {
            DataType::Forest => self.draw_areas(pixmap, &collection.features, colors::FOREST, None),
            DataType::Water => self.draw_water(pixmap, &collection.features),
            DataType::Building => self.draw_areas(
                pixmap,
                &collection.features,
                colors::BUILDING,
                Some((colors::BUILDING_OUTLINE, self.px(0.5, 0.7))),
            ),
            DataType::Road => self.draw_roads(pixmap, &collection.features),
            DataType::Height => {}
        }
    }

    fn line_path(&self, points: &[Point]) -> Option<tiny_skia::Path> {
        let mut builder = PathBuilder::new();
        for (i, p) in points.iter().enumerate() {
            let (x, y) = self.projector.project(*p);
            if i == 0 {
                builder.move_to(x, y);
            } else {
                builder.line_to(x, y);
            }
        }
        builder.finish()
    }

    /// One path with all rings of all polygons, to be filled with even-odd rule (holes).
    fn polygons_path<'a>(&self, rings: impl Iterator<Item = &'a Ring>) -> Option<tiny_skia::Path> {
        let mut builder = PathBuilder::new();
        for ring in rings {
            for (i, p) in ring.iter().enumerate() {
                let (x, y) = self.projector.project(*p);
                if i == 0 {
                    builder.move_to(x, y);
                } else {
                    builder.line_to(x, y);
                }
            }
            builder.close();
        }
        builder.finish()
    }

    fn area_path(&self, geometry: &Geometry) -> Option<tiny_skia::Path> {
        match geometry {
            Geometry::Polygon(rings) => self.polygons_path(rings.iter()),
            Geometry::MultiPolygon(polygons) => self.polygons_path(polygons.iter().flatten()),
            Geometry::LineString(_) => None,
        }
    }

    fn fill(&self, pixmap: &mut Pixmap, path: &tiny_skia::Path, rgb: u32) {
        pixmap.fill_path(
            path,
            &paint(rgb),
            FillRule::EvenOdd,
            Transform::identity(),
            None,
        );
    }

    fn stroke(
        &self,
        pixmap: &mut Pixmap,
        path: &tiny_skia::Path,
        rgb: u32,
        width: f32,
        dash: Option<(f32, f32)>,
    ) {
        let stroke = Stroke {
            width,
            line_cap: if dash.is_some() {
                LineCap::Butt
            } else {
                LineCap::Round
            },
            line_join: LineJoin::Round,
            dash: dash.and_then(|(on, off)| StrokeDash::new(vec![on, off], 0.0)),
            ..Default::default()
        };
        pixmap.stroke_path(path, &paint(rgb), &stroke, Transform::identity(), None);
    }

    fn draw_areas(
        &self,
        pixmap: &mut Pixmap,
        features: &[Feature],
        fill: u32,
        outline: Option<(u32, f32)>,
    ) {
        for feature in features {
            if let Some(path) = self.area_path(&feature.geometry) {
                self.fill(pixmap, &path, fill);
                if let Some((rgb, width)) = outline {
                    self.stroke(pixmap, &path, rgb, width, None);
                }
            }
        }
    }

    fn draw_water(&self, pixmap: &mut Pixmap, features: &[Feature]) {
        self.draw_areas(pixmap, features, colors::WATER, None);
        for feature in features {
            let Geometry::LineString(points) = &feature.geometry else {
                continue;
            };
            let Some(path) = self.line_path(points) else {
                continue;
            };
            let width_m = match feature.tag("waterway") {
                Some("river") => 12.0,
                Some("canal") => 10.0,
                Some("stream" | "tidal_channel") => 3.0,
                Some("ditch" | "drain") => 1.5,
                _ => 2.0,
            };
            self.stroke(pixmap, &path, colors::WATER, self.px(width_m, 1.0), None);
        }
    }

    fn draw_roads(&self, pixmap: &mut Pixmap, features: &[Feature]) {
        let mut roads: Vec<(RoadStyle, tiny_skia::Path)> = vec![];
        for feature in features {
            let Some(class) = feature.tag("highway") else {
                continue;
            };
            match &feature.geometry {
                Geometry::LineString(points) => {
                    if let Some(path) = self.line_path(points) {
                        roads.push((RoadStyle::of(class), path));
                    }
                }
                geometry => {
                    if let Some(path) = self.area_path(geometry) {
                        self.fill(pixmap, &path, colors::PEDESTRIAN_AREA);
                    }
                }
            }
        }
        roads.sort_by_key(|(style, _)| style.rank);

        // Casings first, so that fills of connected roads merge nicely
        for (style, path) in &roads {
            if let Some(casing) = style.casing {
                let width = self.px(style.width_m, style.min_px) + self.px(2.0, 1.0);
                self.stroke(pixmap, path, casing, width, None);
            }
        }
        for (style, path) in &roads {
            let width = self.px(style.width_m, style.min_px);
            let dash = style.dashed.then_some((width * 3.0, width * 2.0));
            self.stroke(pixmap, path, style.fill, width, dash);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RoadStyle {
    rank: u8,
    width_m: f64,
    min_px: f32,
    fill: u32,
    casing: Option<u32>,
    dashed: bool,
}

impl RoadStyle {
    fn of(highway: &str) -> Self {
        let road = |rank, width_m, fill, casing| RoadStyle {
            rank,
            width_m,
            min_px: 1.5,
            fill,
            casing: Some(casing),
            dashed: false,
        };
        let path = |width_m, fill| RoadStyle {
            rank: 0,
            width_m,
            min_px: 1.0,
            fill,
            casing: None,
            dashed: true,
        };
        match highway {
            "motorway" | "motorway_link" => road(10, 14.0, 0xe892a2, 0xdc2a67),
            "trunk" | "trunk_link" => road(9, 12.0, 0xf9b29c, 0xc84e2f),
            "primary" | "primary_link" => road(8, 10.0, 0xfcd6a4, 0xa06b00),
            "secondary" | "secondary_link" => road(7, 9.0, 0xf7fabf, 0x707d05),
            "tertiary" | "tertiary_link" => road(6, 8.0, 0xffffff, 0x8f8f8f),
            "residential" | "unclassified" | "road" => road(5, 6.0, 0xffffff, 0xbbbbbb),
            "living_street" => road(4, 5.0, 0xededed, 0xc4c4c4),
            "pedestrian" => road(3, 4.0, colors::PEDESTRIAN_AREA, 0x999999),
            "service" => road(2, 4.0, 0xffffff, 0xbbbbbb),
            "track" => path(3.0, 0x996600),
            "cycleway" => path(1.5, 0x0000ff),
            "bridleway" => path(1.5, 0x008000),
            "footway" | "path" | "steps" | "corridor" => path(1.5, 0xfa8072),
            _ => road(1, 4.0, 0xffffff, 0xbbbbbb),
        }
    }
}
