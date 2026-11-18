use axum::extract::Path;
use axum::{extract::State, response::IntoResponse};
use oc_geo::region::WorldRegionIndex;
use oc_root::files::File;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetRegionBackgroundPath {
    region: WorldRegionIndex,
}

#[axum::debug_handler]
pub async fn get_region(
    path: Path<GetRegionBackgroundPath>,
    State(state): State<super::State>,
) -> impl IntoResponse {
    let file = File::Region {
        index: path.region.0,
        width: state.config.region_width,
        height: state.config.region_height,
    };
    super::stream(state.files().fs(file)).await
}

#[derive(Debug, Deserialize)]
pub struct GetInteriorPath {
    interior: u32,
}

#[axum::debug_handler]
pub async fn get_interior(
    path: Path<GetInteriorPath>,
    State(state): State<super::State>,
) -> impl IntoResponse {
    super::stream(state.files().fs(File::Interior(path.interior))).await
}

#[axum::debug_handler]
pub async fn get(State(state): State<super::State>) -> impl IntoResponse {
    super::stream(state.files().fs(File::WorldArchive)).await
}
