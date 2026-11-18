use std::{net::SocketAddr, ops::Deref, sync::Arc};

use axum::{Router, routing::get};
use derive_more::Constructor;
use message_io::network::Endpoint;

use crate::{config::ServerConfig, network::NetworkConfig};

mod mod_;
mod sprites;
mod world;

#[derive(Constructor)]
pub struct Static {
    state: Arc<crate::state::State<Endpoint>>,
    network: NetworkConfig,
    config: ServerConfig,
}

#[derive(Clone, Constructor)]
pub struct State {
    pub state: Arc<crate::state::State<Endpoint>>,
    pub _network: NetworkConfig,
    pub config: ServerConfig,
}

impl Static {
    pub fn serve(&self, host: SocketAddr) -> Result<(), std::io::Error> {
        let state = State::new(
            self.state.clone(),
            self.network.clone(),
            self.config.clone(),
        );
        let app = Router::new()
            .route("/mod", get(mod_::get))
            .route("/world", get(world::get))
            .route("/region/{region}", get(world::get_region))
            .route("/interior/{interior}", get(world::get_interior))
            .route("/soldiers_outline", get(sprites::get_soldiers_outline))
            .with_state(state);

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let listener = tokio::net::TcpListener::bind(host).await.unwrap();
            tracing::info!("Serve statics on http://{host}");
            axum::serve(listener, app).await.unwrap();
        });

        Ok(())
    }
}

impl State {
    pub fn files(&self) -> oc_root::files::Files {
        let mod_ = self.world().mod_().canonical();
        let world = self.world().meta().canonical();
        oc_root::files::Files::derived_only(self.config.roots.clone(), &mod_, &world)
    }
}

/// Stream a file as response body
pub async fn stream(path: std::path::PathBuf) -> axum::response::Response {
    use axum::response::IntoResponse;

    match tokio::fs::File::open(&path).await {
        Ok(file) => axum::http::Response::builder()
            .header("Content-Type", "application/octet-stream")
            .body(axum::body::Body::from_stream(
                tokio_util::io::ReaderStream::new(file),
            ))
            .unwrap()
            .into_response(),
        Err(e) => {
            tracing::error!("Can't open static file {}: {e}", path.display());
            axum::http::StatusCode::NOT_FOUND.into_response()
        }
    }
}

impl Deref for State {
    type Target = crate::state::State<Endpoint>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
