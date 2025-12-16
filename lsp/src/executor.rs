use crate::parser::HttpRequest;
use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub duration_ms: u64,
}

pub async fn execute_request(req: &HttpRequest) -> Result<HttpResponse> {
    let start = Instant::now();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Build the request
    let mut request_builder = match req.method.as_str() {
        "GET" => client.get(&req.url),
        "POST" => client.post(&req.url),
        "PUT" => client.put(&req.url),
        "DELETE" => client.delete(&req.url),
        "PATCH" => client.patch(&req.url),
        "HEAD" => client.head(&req.url),
        _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", req.method)),
    };

    // Add headers
    for (name, value) in &req.headers {
        request_builder = request_builder.header(name, value);
    }

    // Add body if present (but not for HEAD requests which can't have bodies)
    if let Some(body) = &req.body {
        if req.method != "HEAD" {
            request_builder = request_builder.body(body.clone());
        }
    }

    // Execute the request
    let response = request_builder.send().await?;

    let duration_ms = start.elapsed().as_millis() as u64;

    // Extract response data
    let status = response.status().as_u16();
    let status_text = response.status().canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    let mut headers = HashMap::new();
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }

    let body = response.text().await?;

    Ok(HttpResponse {
        status,
        status_text,
        headers,
        body,
        duration_ms,
    })
}

impl HttpResponse {
    pub fn format_as_http(&self) -> String {
        let mut result = String::new();

        // Status line
        result.push_str(&format!("HTTP/1.1 {} {}\n", self.status, self.status_text));

        // Headers
        for (name, value) in &self.headers {
            result.push_str(&format!("{}: {}\n", name, value));
        }

        // Empty line before body
        result.push('\n');

        // Body
        result.push_str(&self.body);

        result
    }

    pub fn summary(&self) -> String {
        format!("{} {} ({}ms)", self.status, self.status_text, self.duration_ms)
    }
}
