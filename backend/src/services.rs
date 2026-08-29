use axum::extract::State;

use crate::AppState;




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
