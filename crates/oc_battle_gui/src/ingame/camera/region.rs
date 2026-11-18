use bevy::prelude::*;
use oc_geo::{
    region::{RegionXy, WorldRegionIndex},
    tile::TileXy,
};
use oc_network::ToServer;
use oc_root::{Wcfg, WcfgInto, WorldConfig, y::Y};
use oc_utils::{d2::Xy, let_some};

use crate::{
    config::{Config, Config_},
    ingame::region::{ForgottenRegion, ListeningRegion},
    network::output::ToServerEvent,
};

#[derive(Debug, Event)]
pub struct UpdateRegions(pub Vec2);

#[derive(Debug, Clone)]
pub struct Region(pub WorldRegionIndex);

pub fn on_update_regions(
    point: On<UpdateRegions>,
    w: Res<Wcfg>,
    c: Res<Config>,
    mut commands: Commands,
    mut state: ResMut<super::State>,
) {
    let_some!(w = &w.0, return);
    tracing::trace!(name="update-regions", point=?point.0);
    static EMPTY: Vec<Region> = vec![];
    let current = state.regions.as_ref().unwrap_or(&EMPTY);
    let regions = regions(w, &c, point.0);

    let new: Vec<Region> = regions
        .iter()
        .filter(|region| current.iter().find(|r| r.0 == **region).is_none())
        .map(|i| Region(*i))
        .collect();

    let still: Vec<Region> = current
        .iter()
        .filter_map(|region| {
            regions
                .iter()
                .find(|i| **i == region.0)
                .map(|_| region.clone())
        })
        .collect();

    let old: Vec<WorldRegionIndex> = current
        .iter()
        .filter(|region| regions.iter().find(|i| **i == region.0).is_none())
        .map(|r| r.0)
        .collect();

    for new_ in &new {
        tracing::trace!(name="update-regions-new", region=?new_.0);
        commands.trigger(ToServerEvent(ToServer::ListenRegion(new_.0)));
        commands.trigger(ListeningRegion(new_.0));
    }

    for old_ in &old {
        tracing::trace!(name="update-regions-old", region=?old_);
        commands.trigger(ToServerEvent(ToServer::ForgotRegion(*old_)));
        commands.trigger(ForgottenRegion(*old_));
    }

    let now = [new, still].concat();
    state.regions = Some(now);
}

/// List regions to consider around given point
pub fn regions(w: &WorldConfig, c: &Config_, center: Vec2) -> Vec<WorldRegionIndex> {
    let center = center.to_gui_y(w);
    let center: TileXy = [center.x, center.y].into_(w);
    let center: RegionXy = center.into_(w);

    let from_x_minus = c.regions_width / 2;
    let from_x = center.0.0 - from_x_minus.min(center.0.0);
    let from_y_minus = c.regions_height / 2;
    let from_y = center.0.1 - from_y_minus.min(center.0.1);
    let from = RegionXy(Xy(from_x, from_y));

    let to_x_plus = c.regions_width / 2;
    let to_x = (center.0.0 + to_x_plus).min(w.regions_width() - 1);
    let to_y_plus = c.regions_height / 2;
    let to_y = (center.0.1 + to_y_plus).min(w.regions_height() - 1);
    let to = RegionXy(Xy(to_x, to_y));

    let (from_region_x, from_region_y) = (from.0.0, from.0.1);
    let (to_region_x, to_region_y) = (to.0.0, to.0.1);

    (from_region_x..=to_region_x)
        .flat_map(|x| (from_region_y..=to_region_y).map(move |y| RegionXy(Xy(x, y)).into_(w)))
        .collect()
}
