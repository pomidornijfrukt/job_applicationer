use anyhow::{Context, Result, anyhow};
use rquickjs::{Context as JsContext, Runtime};
use serde_json::Value;

use crate::services::data_extracter::JobPosting;

pub trait FrameworkHandler {
    fn name(&self) -> &'static str;
    fn extract(&self, html: &str) -> Result<Vec<Value>>;
}

pub struct NuxtHandler;
impl NuxtHandler {
    pub fn extract_job(&self, payload: &Value, source: &str) -> Result<JobPosting> {
        let job_data = get_path(payload, &["useState", "job-posting-data"])
            .ok_or_else(|| anyhow!("job-posting-data not found"))?;
        
        let mut job: JobPosting = serde_json::from_value(job_data.clone())
            .context("invalid Nuxt job-posting-data")?;
        job.source_url = source.to_string();

        Ok(job)
    }
}

fn strip_nuxt_metadata(mut payload: Value) -> Value {
    let Some(root) = payload.as_object_mut() else {
        return payload;
    };

    // Nuxt/framework/runtime metadata.
    for key in [
        "layout",
        "data",
        "fetch",
        "error",
        "config",
        "head",
        "_asyncData",
        "_errors",
        "serverRendered",
        "hasViewAccessClass",
        "state",
    ] {
        root.remove(key);
    }

    // Nuxt useState metadata.
    if let Some(Value::Object(use_state)) = root.get_mut("useState") {
        use_state.remove("googleMapApiKey");

        // Paradox job-posting-data.
        if let Some(Value::Object(job_data)) =
            use_state.get_mut("job-posting-data")
        {
            // Remove metadata that is not useful for semantic extraction.
            for key in [
                "ai_info",
                "company_settings",
                "composite_slug",
                "error",
                "invite_token",
                "job_id",
                "page_styles",
                "request_language_code",
                "slug",
            ] {
                job_data.remove(key);
            }

            // Keep company_info.name.
            if let Some(Value::Object(company)) =
                job_data.get_mut("company_info")
            {
                company.remove("id");
                company.remove("logo_url");
            }

            // Remove Paradox/ATS internal identifiers.
            if let Some(Value::Object(job_info)) =
                job_data.get_mut("job_info")
            {
                for key in [
                    "applicant_flow_id",
                    "conversation_id",
                    "job_journey_targeting_id",
                    "job_loc_id",
                    "job_req_id",
                    "job_token",
                    "live_job_id",
                    "posting_job_id",
                    "brand_logo",
                ] {
                    job_info.remove(key);
                }
            }

            // Keep page_settings.site_description.
            if let Some(Value::Object(settings)) =
                job_data.get_mut("page_settings")
            {
                for key in [
                    "candidate_language_on",
                    "geoloc_allowed",
                    "is_ats_job_feed_on",
                    "is_cms_on",
                    "is_flynn_company",
                    "language_code",
                    "multi_branding_on",
                    "origin_url",
                    "redirect_url",
                    "referer_url",
                    "request_language_code",
                    "show_chat_prompt",
                    "show_job_language",
                    "tracking_pixel_script",
                ] {
                    settings.remove(key);
                }
            }
        }
    }

    // Remove empty objects created by the cleanup above.
    remove_empty_objects(&mut payload);

    payload
}

fn remove_empty_objects(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for child in map.values_mut() {
                remove_empty_objects(child);
            }

            map.retain(|_, value| {
                !matches!(value, Value::Object(obj) if obj.is_empty())
            });
        }

        Value::Array(values) => {
            for value in values {
                remove_empty_objects(value);
            }
        }

        _ => {}
    }
}


fn get_path<'the_type>(value: &'the_type Value, path: &[&str]) -> Option<&'the_type Value> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
}

impl FrameworkHandler for NuxtHandler {
    fn name(&self) -> &'static str {
        "nuxt"
    }

    fn extract(&self, html: &str) -> Result<Vec<Value>> {
        let script = find_nuxt_script(html)?;
        let value = evaluate_javascript(&script)?;
        let value = strip_nuxt_metadata(value);

        Ok(vec![value])
    }
}

fn find_nuxt_script(html: &str) -> Result<String> {
    let marker = "window.__NUXT__=";
    let start = html
        .find(marker)
        .ok_or_else(|| anyhow!("Nuxt payload not found"))?;
    let expression_start = start + marker.len();
    let remaining = &html[expression_start..];
    let end = find_js_expression_end(remaining)
        .ok_or_else(|| anyhow!("Could not determine end of Nuxt payload"))?;

    Ok(remaining[..end].trim().to_string())
}

fn find_js_expression_end(input: &str) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut in_string: Option<u8> = None;
    let mut escaped = false;

    for (i, byte) in input.bytes().enumerate() {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
                continue;
            }

            if byte == b'\\' {
                escaped = true;
                continue;
            }

            if byte == quote {
                in_string = None;
            }

            continue;
        }

        match byte {
            b'"' | b'\'' | b'`' => in_string = Some(byte),
            b'(' => paren_depth += 1,
            b')' => {
                if paren_depth == 0 {
                    return None;
                }
                paren_depth -= 1;
            }
            b'[' => bracket_depth += 1,
            b']' => {
                if bracket_depth == 0 {
                    return None;
                }
                bracket_depth -= 1;
            }
            b'{' => brace_depth += 1,
            b'}' => {
                if brace_depth == 0 {
                    return None;
                }
                brace_depth -= 1;
            }
            b';' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return Some(i);
            }
            _ => {}
        }
    }

    if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 {
        Some(input.len())
    } else {
        None
    }
}

fn evaluate_javascript(script: &str) -> Result<Value> {
    let runtime = Runtime::new().context("failed to create JavaScript runtime")?;
    let context = JsContext::full(&runtime).context("failed to create JavaScript context")?;

    context.with(|ctx| -> Result<Value> {
        let wrapped = format!("({script})");
        let value: rquickjs::Value = ctx
            .eval(wrapped.as_bytes())
            .context("failed to evaluate Nuxt payload")?;

        js_value_to_json(ctx, value)
    })
}

fn js_value_to_json<'js>(ctx: rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> Result<Value> {
    if value.is_null() || value.is_undefined() {
        return Ok(Value::Null);
    }

    if let Some(value) = value.as_bool() {
        return Ok(Value::Bool(value));
    }

    if let Some(value) = value.as_int() {
        return Ok(Value::Number(value.into()));
    }

    if let Some(value) = value.as_float() {
        let number = serde_json::Number::from_f64(value)
            .ok_or_else(|| anyhow!("invalid JavaScript number"))?;
        return Ok(Value::Number(number));
    }

    if let Some(value) = value.as_string() {
        return Ok(Value::String(
            value
                .to_string()
                .context("failed to convert JavaScript string")?,
        ));
    }

    if let Some(object) = value.as_object() {
        if let Some(array) = value.as_array() {
            let mut result = Vec::with_capacity(array.len());
            for i in 0..array.len() {
                let item: rquickjs::Value = array
                    .get(i)
                    .context("failed to read JavaScript array item")?;
                result.push(js_value_to_json(ctx.clone(), item)?);
            }
            return Ok(Value::Array(result));
        }

        let keys: Vec<String> = object
            .keys()
            .collect::<rquickjs::Result<Vec<String>>>()
            .context("failed to enumerate JavaScript object")?;
        let mut result = serde_json::Map::new();

        for key in keys {
            let item: rquickjs::Value = object
                .get(&key)
                .with_context(|| format!("failed to read property `{key}`"))?;
            result.insert(key, js_value_to_json(ctx.clone(), item)?);
        }

        return Ok(Value::Object(result));
    }

    Err(anyhow!("unsupported JavaScript value"))
}
