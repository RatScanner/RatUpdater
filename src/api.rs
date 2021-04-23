use anyhow::{anyhow, Result};
use std::time::Duration;
use ureq::Response;

pub fn get_resource(resource: &str) -> Result<String> {
    let response = get_resource_response(resource)?;
    let response = response.into_json::<serde_json::Value>()?;

    let value = response
        .get("value")
        .and_then(|val| val.as_str())
        .ok_or(anyhow!("Invalid API response"))?
        .to_string();

    Ok(value)
}

fn get_resource_response(resource: &str) -> Result<Response> {
    const API_URL: &[&str] = &[
        "https://api.ratscanner.com/v4/res",
        "https://api.ratscanner.com/v3/res",
        "https://api.ratscanner.com/v2/res/",
    ];

    let mut error = None;
    for api_url in API_URL {
        let path = format!("{}{}", api_url, resource);
        match request(&path) {
            Ok(response) => return Ok(response),
            Err(err) => error = Some(err),
        }
    }

    Err(error.unwrap())
}

fn request(path: &str) -> Result<Response> {
    // Make request
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(15))
        .build();
    let response = agent.get(path).call()?;

    Ok(response)
}
