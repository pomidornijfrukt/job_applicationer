use sqlx::PgPool;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod routes;
pub mod services;
mod tools;
#[derive(Clone)]
pub struct AppState {
    _db: PgPool,
    pub http_client: reqwest::Client,
    healthy: bool,
}

#[tokio::main]
async fn main() {
    // degub tracing
    tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("backend=debug,llm=debug,app_error=debug,info")),
    )
    .init();

    let state = AppState {
        _db: PgPool::connect("postgres://postgres:postgres@localhost:5432/postgres")
            .await
            .unwrap_or_else(|err| {
                error!(error = %err, "There is no database there");
                panic!("Failed to connect to database");
            }),
        http_client: reqwest::Client::new(),
        healthy: true,
    };

    let app = crate::routes::create_routes(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    info!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

