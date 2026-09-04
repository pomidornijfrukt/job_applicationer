use axum::{Json, extract::State, http::StatusCode};
use app_error::{AppError, ParsingError};
use scraper::Html;
use llm::chat_llm;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use tracing::{debug, error, info, warn};
use url::Url;

use crate::{
    AppState,
    services::{
        candidates::{collect_candidates, format_path},
        data_extracter::{extract_data, extract_json_ld, find_job_posting, scrape_page},
        per_framework::nuxt::{FrameworkHandler, NuxtHandler},
    },
};

#[derive(Deserialize, Serialize)]
pub struct LinkBody {
    pub link: String,
    pub optional_status: Option<UrlValidationStatus>,
}

#[derive(Serialize)]
pub struct LinkResponse {
    pub link: String,
    pub status: String,
    pub comment: Option<String>, // Optional comment field
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
) -> Result<(StatusCode, Json<LinkResponse>), AppError> {
    match payload.validate_and_normalize(&state.http_client).await {
        UrlValidationStatus::Valid => {
            payload.optional_status = Some(UrlValidationStatus::Valid);
        }

        UrlValidationStatus::Invalid(reason) => {
            payload.optional_status = Some(UrlValidationStatus::Invalid(reason.clone()));
            // bad - tell the user
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(LinkResponse {
                    link: payload.link,
                    status: reason,
                    comment: None,
                }),
            ));
        }

        UrlValidationStatus::Unverified(reason) => {
            // good, but unverified - tell the user
            payload.optional_status = Some(UrlValidationStatus::Unverified(reason.clone()));
            warn!(url = %payload.link, reason = %reason, "URL could not be verified");
        }
    }


    if payload.link.as_str().trim().is_empty() {
        return Err(AppError::Parsing(ParsingError::NoDataError {
            details: "No link provided".to_string(),
        }));
    }

    let status = payload
        .optional_status
        .as_ref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let comment = "comment";

    match scrape_page(state.http_client, payload.link.as_str()).await {
        Ok(response) => {
            let html = response;
            debug!(html = %html, "Received HTML response");
            if html.contains("<html") {
                info!("The response is an HTML page");
                let (_aboba, llm_payloads) = {
                    let document = Html::parse_document(&html);
                    let mut aboba = String::new();
                    let mut llm_payloads = Vec::new();

                if html.contains("window.__NUXT__=") {
                    let nuxt_handler = NuxtHandler;
                    match nuxt_handler.extract(&html) {
                        Ok(payloads) => {
                            for nuxt_payload in &payloads {
                                let job = nuxt_handler.extract_job(nuxt_payload, payload.link.as_str())?;

                                debug!(?job, "Extracted job from Nuxt payload");
                                for candidate in collect_candidates(nuxt_payload) {
                                    aboba.push_str(&format!(
                                        "Nuxt candidate {} = {:?}",
                                        format_path(&candidate.path),
                                        candidate.value
                                    ));
                                    aboba.push('\n');
                                }
                            };
                            llm_payloads = payloads;
                        }
                        Err(err) => warn!(error = %err, "Could not extract Nuxt payload"),
                    }
                }

                // SCRAPPING THE LD
                match extract_json_ld(&document) {
                    Ok(json_ld) => {
                        debug!(?json_ld, "Extracted JSON-LD job data");
                        match find_job_posting(&json_ld) {
                            Some(job_posting_value) => {
                                let job_posting = extract_data(job_posting_value, payload.link.clone())?;
                                debug!(
                                    company = ?job_posting.company,
                                    technologies = ?job_posting.technologies,
                                    compensation = ?job_posting.compensation,
                                    job_title = ?job_posting.job_title,
                                    location = ?job_posting.location,
                                    duration = ?job_posting.duration,
                                    date_posted = ?job_posting.date_posted,
                                    "Extracted job posting fields"
                                );
                            }
                            None => {
                                warn!("No valid JobPosting JSON-LD block found; continuing with other extraction");
                            }
                        }
                    }
                    Err(err) => {
                        warn!(error = %err, "Could not extract JSON-LD");
                    }
                }

                if llm_payloads.is_empty() {
                    warn!("No LLM payloads extracted; going with just blank html");
                    llm_payloads = match scrape_html(&html) {
                        Ok(llm_payloads) => llm_payloads,
                        Err(err) => {
                            error! (error = %err, "Could not scrape HTML for LLM payloads");
                            Vec::new()
                        }
                    };
                }

                    (aboba, llm_payloads)
                };
                match chat_llm(llm_payloads).await{
                    Ok(_) => {
                        info!("LLM returned an answer that is not collected yet");
                    }
                    Err(err) => {
                        error!(error = %err, "Error while chatting with LLM");
                    }
                };
            } else {
                warn!("The response is not an HTML page");
            }
        }
        Err(error) => {
            error!(error = %error, "Could not reach URL");
        }
    };

    info!(link = %payload.link, "Link parse endpoint hit");
    Ok((
        StatusCode::OK,
        Json(LinkResponse {
            link: payload.link,
            status,
            comment: Some(comment.to_string()), // Add the comment to the response
        }),
    ))
}

fn scrape_html(html: &str) -> Result<Vec<Value>, AppError> {
    if html.contains("<html") {
        let document = Html::parse_document(html);
        extract_json_ld(&document)
    } else {
        Err(AppError::Parsing(ParsingError::InvalidFormat {
            expected: "HTML document",
            actual: "Response is not an HTML document".to_string(),
        }))
    }
}