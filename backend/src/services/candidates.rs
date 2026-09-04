use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Candidate {
    pub path: Vec<String>,
    pub value: Value,
}

pub fn collect_candidates(value: &Value) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    collect_value(value, &mut Vec::new(), &mut candidates);
    candidates
}

fn collect_value(value: &Value, path: &mut Vec<String>, candidates: &mut Vec<Candidate>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                path.push(key.clone());
                collect_value(child, path, candidates);
                path.pop();
            }
        }
        Value::Array(array) => {
            for (index, child) in array.iter().enumerate() {
                path.push(index.to_string());
                collect_value(child, path, candidates);
                path.pop();
            }
        }
        _ if is_candidate(value) => candidates.push(Candidate {
            path: path.clone(),
            value: value.clone(),
        }),
        _ => {}
    }
}

fn is_candidate(value: &Value) -> bool {
    match value {
        Value::String(value) => is_candidate_string(value),
        Value::Number(number) => {
            number
                .as_i64()
                .is_some_and(|value| value.abs() > 1)
                || number
                    .as_u64()
                    .is_some_and(|value| value > 1)
                || number.as_f64().is_some_and(|value| value.abs() > 1.0)
        }
        Value::Null | Value::Bool(_) => false,
        Value::Array(_) | Value::Object(_) => false,
    }
}

fn is_candidate_string(value: &str) -> bool {
    let value = value.trim();

    if value.is_empty() || value.len() > 4_000 {
        return false;
    }

    let lower = value.to_ascii_lowercase();
    let looks_like_url = lower.starts_with("http://") || lower.starts_with("https://");
    let looks_like_code = lower.starts_with("function ")
        || lower.starts_with("(()")
        || lower.starts_with("webpack")
        || lower.contains("sourceMappingURL=");
    let looks_like_css = lower.contains("{display:")
        || lower.contains("{ margin:")
        || lower.contains("font-family:");

    !looks_like_url && !looks_like_code && !looks_like_css
}

pub fn format_path(path: &[String]) -> String {
    path.join(".")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{collect_candidates, format_path};

    #[test]
    fn flattens_nested_values_and_filters_obvious_noise() {
        let value = json!({
            "state": {
                "title": "Software Engineer",
                "enabled": true,
                "requestId": 1,
                "salary": 120000,
                "link": "https://example.com/jobs/1",
                "description": "We are looking for an engineer."
            }
        });

        let candidates = collect_candidates(&value);
        let paths: Vec<String> = candidates
            .iter()
            .map(|candidate| format_path(&candidate.path))
            .collect();

        assert_eq!(paths, ["state.title", "state.salary", "state.description"]);
    }
}
