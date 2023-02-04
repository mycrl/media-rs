use tower_http::cors::CorsLayer;
use axum::http::{
    HeaderValue,
    StatusCode,
    Response,
};

use axum::{
    response::IntoResponse,
    routing::get,
    Router,
};

use axum::extract::{
    ConnectInfo,
    State,
    Path,
};

use crate::{
    config::HttpFlv,
    proto::http::*,
    router,
};

use std::{
    net::SocketAddr,
    sync::Arc,
};

async fn fork_socket(
    Path(name): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(router): State<Arc<router::Router>>,
) -> impl IntoResponse {
    log::info!("http flv connection name: {}, addr: {}", name, addr);

    if let Some(reader) = router.get_receiver(&addr, &name).await {
        Response::new(Stream::new(reader)).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn run(
    cfg: HttpFlv,
    router: Arc<router::Router>,
) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/:name", get(fork_socket))
        .layer(CorsLayer::new().allow_origin(
            cfg.allow_origin.as_str().parse::<HeaderValue>().unwrap(),
        ))
        .with_state(router)
        .into_make_service_with_connect_info::<SocketAddr>();
    axum::Server::bind(&cfg.listen).serve(app).await?;
    Ok(())
}
