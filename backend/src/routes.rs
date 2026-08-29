use crate::{
    AppState, 
    tools::create_both_routes,
    services::{about, create_user, index, health_check, list_users},
};

pub fn create_routes(state: AppState) -> axum::Router {
    let routes = axum::Router::new()
        .merge(create_both_routes("/troll/about", about))
        .merge(create_both_routes("/troll/users", list_users))
        .merge(create_both_routes("/troll/users/create", create_user));
    let app = axum::Router::new()
        .merge(create_both_routes("/health", health_check))
        .with_state(state.clone())
        .merge(create_both_routes("/troll", index))
        .merge(routes);
    app.with_state(state)
}
