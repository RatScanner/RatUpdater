use anyhow::{anyhow, bail, Result};
use ureq::Response;

pub fn get_resource(resource: &str) -> Result<String> {
    let response = get_resource_response(resource)?;
    let response = response.into_json()?;

    let value = response
        .get("value")
        .and_then(|val| val.as_str())
        .ok_or(anyhow!("Invalid API response"))?
        .to_string();

    Ok(value)
}

fn get_resource_response(resource: &str) -> Result<Response> {
    const API_URL: &[&str] = &[
        "http://ratscanner.com:8080/api/v2/res/",
        "https://api.ratscanner.com/v2/res/",
        "https://ratscanner.com/api/v2/res/",
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
    let response = ureq::get(path)
        .timeout_connect(15_000)
        .timeout_read(15_000)
        .call();

    // Check for synthetic error
    if response.synthetic() {
        let error = response.into_synthetic_error().unwrap();
        return Err(error.into());
    }

    // Check for other errors
    if response.error() {
        bail!("Received tatus code {}", response.status());
    }

    Ok(response)
}
