use crate::{
    AppState,
    services::all_services::{
        create_user, health_check, list_users, index
    },
    services::link::link_parse,
    tools::{create_both_get_routes, create_both_post_routes},
};

pub fn create_routes(state: AppState) -> axum::Router {
    let random_routes_lol = axum::Router::new()
        .merge(create_both_get_routes("/troll/users", list_users))
        .merge(create_both_get_routes("/troll/users/create", create_user))
        .merge(create_both_get_routes("/troll", index));

    let real_routes = axum::Router::new().merge(create_both_post_routes("/link", link_parse));
    let app = axum::Router::new()
        .merge(create_both_get_routes("/health", health_check))
        .merge(random_routes_lol)
        .merge(real_routes)
        .with_state(state.clone());

    app.with_state(state)
}
