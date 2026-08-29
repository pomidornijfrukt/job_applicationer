use sqlx::PgPool;

mod routes;
mod services;
mod tools;
#[derive(Clone)]
pub struct AppState {
    _db: PgPool,
    pub http_client: reqwest::Client,
    healthy: bool,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        _db: PgPool::connect("postgres://postgres:postgres@localhost:5432/postgres")
            .await
            .unwrap_or_else(|err| {
                println!("There is no database there:\n{err}");
                panic!("Failed to connect to database");
            }),
        http_client: reqwest::Client::new(),
        healthy: true,
    };

    let app = crate::routes::create_routes(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

