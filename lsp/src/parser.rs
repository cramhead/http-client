use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub line_number: usize,
}

pub fn parse_http_file(content: &str) -> Vec<HttpRequest> {
    let mut requests = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

    let mut current_block_start: Option<usize> = None;

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check if this line starts with ###
        if trimmed.starts_with("###") {
            // If we have a current block, parse it
            if let Some(start) = current_block_start {
                if let Some(request) = parse_block_lines(&lines, start, line_idx) {
                    requests.push(request);
                }
            }
            // Start new block after this delimiter
            current_block_start = Some(line_idx + 1);
        }
    }

    // Don't forget the last block
    if let Some(start) = current_block_start {
        if let Some(request) = parse_block_lines(&lines, start, lines.len()) {
            requests.push(request);
        }
    } else if !lines.is_empty() {
        // No ### delimiters, parse entire file as one block
        if let Some(request) = parse_block_lines(&lines, 0, lines.len()) {
            requests.push(request);
        }
    }

    requests
}

fn parse_block_lines(lines: &[&str], start_idx: usize, end_idx: usize) -> Option<HttpRequest> {
    let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

    let mut method = String::new();
    let mut url = String::new();
    let mut headers = HashMap::new();
    let mut body_lines = Vec::new();
    let mut request_line_number: Option<usize> = None;
    let mut in_body = false;

    for idx in start_idx..end_idx {
        let line = lines[idx];
        let trimmed = line.trim();

        // Skip empty lines before finding the request
        if request_line_number.is_none() && trimmed.is_empty() {
            continue;
        }

        // Skip comments
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Try to find HTTP request line
        if request_line_number.is_none() {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let potential_method = parts[0].to_uppercase();
                if valid_methods.contains(&potential_method.as_str()) {
                    method = potential_method;
                    url = parts[1].to_string();
                    request_line_number = Some(idx);
                    continue;
                }
            }
        } else if in_body {
            // Collect body lines
            body_lines.push(line);
        } else if trimmed.is_empty() {
            // Empty line marks start of body
            in_body = true;
        } else if let Some(colon_idx) = trimmed.find(':') {
            // Parse header
            let name = trimmed[..colon_idx].trim().to_string();
            let value = trimmed[colon_idx + 1..].trim().to_string();
            headers.insert(name, value);
        }
    }

    request_line_number.map(|line_num| {
        let body = if body_lines.is_empty() {
            None
        } else {
            Some(body_lines.join("\n").trim().to_string())
        };

        HttpRequest {
            method,
            url,
            headers,
            body,
            line_number: line_num,
        }
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_get() {
        let content = "GET http://example.com/api\nAccept: application/json";
        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].url, "http://example.com/api");
        assert_eq!(requests[0].headers.get("Accept"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_parse_post_with_body() {
        let content = r#"POST http://example.com/api
Content-Type: application/json

{"name": "test"}"#;

        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "POST");
        assert_eq!(requests[0].body, Some(r#"{"name": "test"}"#.to_string()));
    }

    #[test]
    fn test_parse_multiple_requests() {
        let content = r#"GET http://example.com/api/1

###

POST http://example.com/api/2
Content-Type: application/json

{"data": "value"}"#;

        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[1].method, "POST");
    }
}
