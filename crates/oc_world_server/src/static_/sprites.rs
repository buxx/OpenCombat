use axum::{extract::State, response::IntoResponse};
use oc_root::files::File;

#[axum::debug_handler]
pub async fn get_soldiers_outline(State(state): State<super::State>) -> impl IntoResponse {
    super::stream(state.files().fs(File::IndividualOutline)).await
}
