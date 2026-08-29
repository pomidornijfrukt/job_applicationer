use axum::{Router, handler::Handler, routing::get};


pub fn create_both_routes<Function, InputVars, AppState>(
    path: &str,
    func: Function,
) -> Router<AppState>
where
    Function: Handler<InputVars, AppState> + Clone + Send + Sync + 'static,
    InputVars: 'static,
    AppState: Clone + Send + Sync + 'static,
{
    let path_with_slash = format!("{path}/");

    Router::new()
        .route(path, get(func.clone()))
        .route(&path_with_slash, get(func))
}