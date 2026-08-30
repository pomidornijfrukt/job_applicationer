use chrono::NaiveDate;
use regex::Regex;

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

#[derive(Debug)]
enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Internship,
    Temporary,
    Volunteer,
    PerDiem,
    Other,
}

pub struct JobPosting {
    pub job_title: Option<String>,
    pub company: Option<String>,
    pub location: Option<Location>,
    pub description: Option<String>,
    pub technologies: Vec<String>,

    pub compensation: Option<Compensation>,
    pub duration: Option<Duration>,

    pub date_posted: Option<NaiveDate>,
    pub valid_through: Option<NaiveDate>,
    pub employment_type: Option<EmploymentType>,

    pub source_url: String,
}

#[derive(Debug)]
pub struct Duration {
    minimum: Option<u32>,
    maximum: Option<u32>,
    unit: DurationUnit,
}

#[derive(Debug)]
pub enum DurationUnit {
    Weeks,
    Months,
    Years,
}

#[derive(Debug)]
pub struct Compensation {
    min: Option<i64>,
    max: Option<i64>,
    currency: Option<String>,
    period: Option<PayPeriod>,
}

#[derive(Debug)]
pub enum PayPeriod {
    Hour,
    Day,
    Week,
    Month,
    Year,
    Unknown,
}

#[derive(Debug)]
pub struct Location {
    street: Option<String>,
    city: Option<String>,
    state: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    remote: bool,
}
use scraper::{Html, Selector};
use serde_json::Value;

use crate::errors::{AppError, ParsingError};

pub fn extract_technologies(description: &str) -> Vec<String> {
    TECHNOLOGIES
        .iter()
        .filter(|technology| {
            let pattern = format!(
                r"(?i)(^|[^\w+#.]){}($|[^\w+#.])",
                regex::escape(technology)
            );

            Regex::new(&pattern)
                .unwrap()
                .is_match(description)
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
    let selector = match Selector::parse(r#"script[type="application/ld+json"]"#) {
        Ok(selector) => selector,
        Err(_) => {
            return Err(AppError::Parsing(ParsingError::InvalidFormat {
                expected: "script[type=\"application/ld+json\"]",
                actual: "failed to parse selector".to_string(),
            }));
        }
    };

    let mut values = Vec::new();

    for element in document.select(&selector) {
        let json = element.text().collect::<String>();
        let parsed = match serde_json::from_str::<Value>(&json) {
            Ok(value) => value,
            Err(err) => {
                return Err(AppError::Parsing(ParsingError::InvalidFormat {
                    expected: "valid JSON in LD+JSON script",
                    actual: err.to_string(),
                }));
            }
        };
        values.push(parsed);
    }

    Ok(values)
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
    let technologies = match &description {
        Some(description) => extract_technologies(description),
        None => Vec::new(),
    };

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
        technologies,
        compensation: None,
        duration: None,
        date_posted: None,
        valid_through: None,
        employment_type: None,
        source_url,
    })
}

