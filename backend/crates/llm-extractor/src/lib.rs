mod llm_client;

use core::str;
use anyhow::{Context, Result as AnyhowResult};
use app_error::{AppError, LLMError};
use llm_client::LlmClient;
use serde_json::Value;

fn build_prompt(blob: &str) -> String {
    format!(
        r##"
Extract the job information.

Return ONLY valid JSON.
Copy information from the input.
If the input does not explicitly contain a value, use null.

{{
  "job_title": null,
  "company": null,
  "location": null,
  "description": null,
  "technologies": null,
  "compensation": null,
  "duration": null,
  "date_posted": null,
  "valid_through": null,
  "employment_type": null
}}

Input:
{blob}

Output:
"##,
    )
}
fn prepare_for_llm(values: Vec<Value>) -> AnyhowResult<String> {
    let value = match values.len() {
        0 => Value::Null,
        1 => values.into_iter().next().unwrap(),
        _ => Value::Array(values),
    };

    serde_json::to_string(&value)
        .context("failed to serialize Nuxt payload")
}
pub async fn chat_llm(values: Vec<Value>) -> Result<(), AppError> {
    let llm = LlmClient::new("http://127.0.0.1:8080");

    let blob = prepare_for_llm(values).context("failed to prepare Nuxt payload for LLM")?;

    let prompt = build_prompt(&blob);

    println!("\n--- PROMPT ---\n");
    println!("{prompt}");
    let answer = match llm.chat(&prompt).await {
        Ok(answer) => answer,
        Err(error) => {
            println!("Error while chatting with LLM: {error}");
            return Err(AppError::LLM(LLMError::FailedChatError { reason: error.to_string() }));
        }
    };

    println!("\n--- LLM ANSWER ---\n");
    println!("{answer}");

    let normalization_prompt = build_normalization_prompt(&answer);
    println!("\n--- NORMALIZATION PROMPT ---\n");
    println!("{normalization_prompt}");

    let normalized_answer = llm.chat(&normalization_prompt).await?;

    println!("\n--- NORMALIZED ANSWER ---\n");
    println!("{normalized_answer}");

    Ok(())
}




fn build_normalization_prompt(job_json: &str) -> String {
    format!(
        r##"
Normalize this extracted job data.

Return ONLY valid JSON.
Use exactly the fields shown below.
Do not add fields.
Do not invent or infer values.
If a value cannot be converted safely, use null.

{{
  "job_title": null,
  "company": null,
  "location": null,
  "description": null,
  "technologies": null,
  "compensation": null,
  "duration": null,
  "date_posted": null,
  "valid_through": null,
  "employment_type": null
}}

Rules:
- technologies must be an array of strings.
- date_posted must be YYYY-MM-DD or null.
- valid_through must be YYYY-MM-DD or null.
- duration must describe an employment duration, not a work arrangement.
- employment_type must describe the employment relationship, not a work arrangement.
- "Hybrid" is not an employment type or duration.
- Preserve strings when they cannot be safely normalized.

Extracted data:
{job_json}

Output:
"##,
    )
}