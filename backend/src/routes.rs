use crate::{
    AppState, services::{about, create_user, health_check, index, link_parse, list_users}, tools::{create_both_get_routes, create_both_post_routes},
};

pub fn create_routes(state: AppState) -> axum::Router {
    let random_routes_lol = axum::Router::new()
        .merge(create_both_get_routes("/troll/about", about))
        .merge(create_both_get_routes("/troll/users", list_users))
        .merge(create_both_get_routes("/troll/users/create", create_user))
        .merge(create_both_get_routes("/troll", index));

    let real_routes = axum::Router::new()
        .merge(create_both_post_routes("/link", link_parse));

    let app = axum::Router::new()
        .merge(create_both_get_routes("/health", health_check))
        .with_state(state.clone())
        .merge(random_routes_lol)
        .merge(real_routes);

    app.with_state(state)
}

