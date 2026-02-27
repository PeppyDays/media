mod container;
mod routes;

use axum::Router;
use axum::routing::post;
use foundation::common::tracing::init_tracing;
use foundation::config::Config;

#[tokio::main]
async fn main() {
    let config = Config::load();

    init_tracing(config.tracing.log_level, config.tracing.log_format);

    let state = container::AppState::build(&config).await;

    let addr = format!("0.0.0.0:{}", config.server.port);
    let app = Router::new()
        .route(
            "/api/image/v1/ingest/create-presigned-url",
            post(routes::image::ingest::create_presigned_url),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
