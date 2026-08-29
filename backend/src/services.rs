use serde::Deserialize;
use axum::{Json, extract::State};

use crate::AppState;

#[derive(Deserialize)]
pub struct LinkBody {
    pub link: String,
}

pub async fn link_parse(Json(payload): Json<LinkBody>) -> String {
    println!("Link parse endpoint hit with link: {}", payload.link);
    format!("Hi!, got a \n{}", payload.link)

}

pub async fn index() -> &'static str {
    "Home"
}
pub async fn about() -> &'static str {
    "About"
}

pub async fn list_users() -> &'static str {
    "List users"
}
pub async fn create_user() -> &'static str {
    "Create user"
}


pub async fn health_check(State(state): State<AppState>) -> &'static str {
    if state.healthy {
        "Healthy"
    } else {
        "Unhealthy"
    }
}
