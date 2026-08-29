use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use url::Url;
use std::fmt;
use crate::AppState;

#[derive(Deserialize, Serialize)]
pub struct LinkBody {
    pub link: String,
    pub optional_status: Option<UrlValidationStatus>,
}

#[derive(Serialize)]
pub struct LinkResponse {
    pub link: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum UrlValidationStatus {
    Valid,
    Invalid(String),
    Unverified(String),
}

impl fmt::Display for UrlValidationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlValidationStatus::Valid => write!(f, "Valid"),
            UrlValidationStatus::Invalid(reason) => write!(f, "Invalid: {reason}"),
            UrlValidationStatus::Unverified(reason) => {
                write!(f, "Unverified: {reason}")
            }
        }
    }
}

impl LinkBody {
    async fn validate_and_normalize(&mut self, client: &reqwest::Client) -> UrlValidationStatus {
        let link = self.link.trim();
        if link.is_empty() {
            return UrlValidationStatus::Invalid("Link is empty".to_string());
        }

        if !link.contains('.') {
            return UrlValidationStatus::Invalid("Link is not a valid URL".to_string());
        }

        let normalized = if link.starts_with("https://") || link.starts_with("https://") {
            link.to_string()
        } else {
            format!("https://{link}")
        };
        let url = match Url::parse(&normalized) {
            Ok(url) => url,
            Err(_) => {
                return UrlValidationStatus::Invalid("URL is not valid".to_string());
            }
        };

        if url.host_str().is_none() {
            return UrlValidationStatus::Invalid("URL does not have a valid host".to_string());
        }

        // Check the exact URL the user provided.
        let response = match client.get(url.as_str()).send().await {
            Ok(response) => response,
            Err(_) => {
                return UrlValidationStatus::Unverified("Could not reach URL".to_string());
            }
        };

        // 2xx means the exact resource responded successfully.
        match response.status() {
            status if status.is_success() => {
                self.link = url.to_string();
                UrlValidationStatus::Valid
            }

            reqwest::StatusCode::NOT_FOUND => {
                UrlValidationStatus::Invalid("The requested resource was not found".to_string())
            }

            _ => {
                self.link = url.to_string();
                UrlValidationStatus::Unverified(
                    "The URL responded, but could not be verified".to_string(),
                )
            }
        }
    }
}

pub async fn link_parse(
    State(state): State<AppState>,
    Json(mut payload): Json<LinkBody>,
) -> (StatusCode, Json<LinkResponse>) {
    match payload.validate_and_normalize(&state.http_client).await {
        UrlValidationStatus::Valid => {
            payload.optional_status = Some(UrlValidationStatus::Valid);
        }

        UrlValidationStatus::Invalid(reason) => {
            payload.optional_status = Some(UrlValidationStatus::Invalid(reason.clone()));
            // bad - tell the user
            return (
                StatusCode::BAD_REQUEST,
                Json(LinkResponse {
                    link: payload.link,
                    status: reason,
                }),
            );
        }

        UrlValidationStatus::Unverified(reason) => {
            // good, but unverified - tell the user
            payload.optional_status = Some(UrlValidationStatus::Unverified(reason.clone()));
            println!("URL {0} could not be verified: {reason}", payload.link);
        }
    }

    if payload.link.as_str().trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(LinkResponse {
                link: payload.link,
                status: "No link provided".to_string(),
            }),
        );
    }

    let status = payload
        .optional_status
        .as_ref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    println!("Link parse endpoint hit with link: {}", payload.link);
    (
        StatusCode::OK,
        Json(LinkResponse {
            link: payload.link,
            status,
        }),
    )
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
