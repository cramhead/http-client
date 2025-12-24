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
    let status_text = response
        .status()
        .canonical_reason()
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
    #[cfg(test)]
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
        format!(
            "{} {} ({}ms)",
            self.status, self.status_text, self.duration_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn create_test_response(status: u16, status_text: &str, duration_ms: u64) -> HttpResponse {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("content-length".to_string(), "42".to_string());

        HttpResponse {
            status,
            status_text: status_text.to_string(),
            headers,
            body: r#"{"message": "success"}"#.to_string(),
            duration_ms,
        }
    }

    #[rstest]
    #[case(200, "OK", 150, "200 OK (150ms)")]
    #[case(201, "Created", 200, "201 Created (200ms)")]
    #[case(400, "Bad Request", 50, "400 Bad Request (50ms)")]
    #[case(404, "Not Found", 75, "404 Not Found (75ms)")]
    #[case(
        500,
        "Internal Server Error",
        1000,
        "500 Internal Server Error (1000ms)"
    )]
    fn test_response_summary(
        #[case] status: u16,
        #[case] status_text: &str,
        #[case] duration_ms: u64,
        #[case] expected: &str,
    ) {
        let response = create_test_response(status, status_text, duration_ms);
        assert_eq!(response.summary(), expected);
    }

    #[test]
    fn test_format_as_http_contains_status_line() {
        let response = create_test_response(200, "OK", 100);
        let formatted = response.format_as_http();

        assert!(formatted.starts_with("HTTP/1.1 200 OK\n"));
    }

    #[test]
    fn test_format_as_http_contains_headers() {
        let response = create_test_response(200, "OK", 100);
        let formatted = response.format_as_http();

        assert!(formatted.contains("content-type: application/json"));
        assert!(formatted.contains("content-length: 42"));
    }

    #[test]
    fn test_format_as_http_contains_body() {
        let response = create_test_response(200, "OK", 100);
        let formatted = response.format_as_http();

        assert!(formatted.contains(r#"{"message": "success"}"#));
    }

    #[test]
    fn test_format_as_http_structure() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/plain".to_string());

        let response = HttpResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: "Hello World".to_string(),
            duration_ms: 50,
        };

        let formatted = response.format_as_http();
        let lines: Vec<&str> = formatted.lines().collect();

        assert_eq!(lines[0], "HTTP/1.1 200 OK");
        assert!(lines
            .iter()
            .any(|line| line.contains("content-type: text/plain")));
        assert!(formatted.ends_with("Hello World"));
    }

    #[test]
    fn test_format_as_http_empty_body() {
        let mut headers = HashMap::new();
        headers.insert("content-length".to_string(), "0".to_string());

        let response = HttpResponse {
            status: 204,
            status_text: "No Content".to_string(),
            headers,
            body: String::new(),
            duration_ms: 30,
        };

        let formatted = response.format_as_http();

        assert!(formatted.starts_with("HTTP/1.1 204 No Content\n"));
        assert!(formatted.contains("content-length: 0"));
    }

    #[test]
    fn test_format_as_http_multiple_headers() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("x-request-id".to_string(), "abc123".to_string());
        headers.insert("cache-control".to_string(), "no-cache".to_string());

        let response = HttpResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: "{}".to_string(),
            duration_ms: 100,
        };

        let formatted = response.format_as_http();

        assert!(formatted.contains("content-type: application/json"));
        assert!(formatted.contains("x-request-id: abc123"));
        assert!(formatted.contains("cache-control: no-cache"));
    }

    #[test]
    fn test_response_with_json_body() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let json_body = r#"{
  "id": 1,
  "name": "Test User",
  "email": "test@example.com"
}"#;

        let response = HttpResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: json_body.to_string(),
            duration_ms: 120,
        };

        let formatted = response.format_as_http();

        assert!(formatted.contains("HTTP/1.1 200 OK"));
        assert!(formatted.contains(r#""name": "Test User""#));
        assert_eq!(response.summary(), "200 OK (120ms)");
    }
}
