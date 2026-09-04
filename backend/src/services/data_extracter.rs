use chrono::NaiveDate;
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::Value;

use app_error::{AppError, ParsingError};

const TECHNOLOGIES: &[&str] = &[
    "Rust",
    "Python",
    "Java",
    "JavaScript",
    "TypeScript",
    "C",
    "C++",
    "C#",
    "Go",
    "Ruby",
    "PHP",
    "Kotlin",
    "Swift",
    "React",
    "Angular",
    "Vue",
    "Vue3",
    "Next.js",
    "Node.js",
    "Express",
    "Flask",
    "Django",
    "FastAPI",
    "Streamlit",
    "SQL",
    "PostgreSQL",
    "MySQL",
    "MongoDB",
    "Redis",
    "Docker",
    "Kubernetes",
    "Terraform",
    "AWS",
    "Azure",
    "GCP",
    "Google Cloud",
    "Git",
    "Linux",
    "pandas",
    "NumPy",
    "scikit-learn",
    "PyTorch",
    "TensorFlow",
    "OpenAI",
];

#[derive(Debug, Deserialize)]
pub enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Internship,
    Temporary,
    Volunteer,
    PerDiem,
    Other,
}

#[derive(Debug, Deserialize)]
pub struct JobPosting {
    pub job_title: Option<String>,
    pub company: Option<String>,
    pub location: Option<Location>,
    pub description: Option<String>,
    pub technologies: Option<Vec<String>>,
    pub compensation: Option<Compensation>,
    pub duration: Option<Duration>,
    pub date_posted: Option<NaiveDate>,
    pub valid_through: Option<NaiveDate>,
    pub employment_type: Option<EmploymentType>,
    // ignores the source_string from deserialization, sets it to an empty one
    #[serde(default)]
    pub source_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Duration {
    minimum: Option<u32>,
    maximum: Option<u32>,
    unit: DurationUnit,
}

#[derive(Debug, Deserialize)]
pub enum DurationUnit {
    Weeks,
    Months,
    Years,
}

#[derive(Debug, Deserialize)]
pub struct Compensation {
    min: Option<i64>,
    max: Option<i64>,
    currency: Option<String>,
    period: Option<PayPeriod>,
}

#[derive(Debug, Deserialize)]
pub enum PayPeriod {
    Hour,
    Day,
    Week,
    Month,
    Year,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct Location {
    street: Option<String>,
    city: Option<String>,
    state: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    remote: bool,
}

pub fn extract_technologies(description: &str) -> Vec<String> {
    TECHNOLOGIES
        .iter()
        .filter(|technology| {
            let pattern = format!(r"(?i)(^|[^\w+#.]){}($|[^\w+#.])", regex::escape(technology));

            Regex::new(&pattern).unwrap().is_match(description)
        })
        .map(|technology| technology.to_string())
        .collect()
}

pub async fn scrape_page(client: reqwest::Client, url: &str) -> Result<String, reqwest::Error> {
    client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await
}

pub fn extract_json_ld(document: &Html) -> Result<Vec<Value>, AppError> {
    let selector = Selector::parse(r#"script[type="application/ld+json"]"#).map_err(|_| {
        AppError::Parsing(ParsingError::InvalidFormat {
            expected: "script[type=\"application/ld+json\"]",
            actual: "failed to parse selector".to_string(),
        })
    })?;

    document
        .select(&selector)
        .map(|element| {
            let json = element.text().collect::<String>();
            serde_json::from_str::<Value>(&json).map_err(|err| {
                AppError::Parsing(ParsingError::InvalidFormat {
                    expected: "valid JSON in LD+JSON script",
                    actual: err.to_string(),
                })
            })
        })
        .collect()
}

pub fn find_job_posting(items: &[Value]) -> Option<&Value> {
    items
        .iter()
        .find(|value| value.get("@type").and_then(Value::as_str) == Some("JobPosting"))
}

pub fn extract_data(value: &Value, source_url: String) -> Result<JobPosting, AppError> {
    let job_title = value
        .get("title")
        .and_then(Value::as_str)
        .map(str::to_string);

    let description = value
        .get("description")
        .and_then(Value::as_str)
        .map(str::to_string);

    // Technologies depend on the description,
    // so extract them after we have the description.
    let technologies = description
        .as_deref()
        .map(extract_technologies)
        .unwrap_or_default();
    let company = value
        .get("hiringOrganization")
        .and_then(|org| org.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);

    // TODO: extract location
    // TODO: extract compensation
    // TODO: extract duration
    // TODO: extract date_posted
    // TODO: extract valid_through
    // TODO: extract employment_type

    Ok(JobPosting {
        job_title,
        company,
        location: None,
        description,
        technologies: Some(technologies),
        compensation: None,
        duration: None,
        date_posted: None,
        valid_through: None,
        employment_type: None,
        source_url,
    })
}
