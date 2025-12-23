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
    let _valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

    let mut current_block_start: Option<usize> = Some(0);

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check if this line starts with ###
        if trimmed.starts_with("###") {
            // Parse the current block up to this delimiter
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
    use rstest::rstest;

    #[rstest]
    #[case("GET http://example.com/api", "GET", "http://example.com/api", 0, None)]
    #[case("get http://example.com/api", "GET", "http://example.com/api", 0, None)]
    #[case("POST http://example.com/api", "POST", "http://example.com/api", 0, None)]
    #[case("PUT http://example.com/api", "PUT", "http://example.com/api", 0, None)]
    #[case("DELETE http://example.com/api", "DELETE", "http://example.com/api", 0, None)]
    #[case("PATCH http://example.com/api", "PATCH", "http://example.com/api", 0, None)]
    #[case("HEAD http://example.com/api", "HEAD", "http://example.com/api", 0, None)]
    #[case("OPTIONS http://example.com/api", "OPTIONS", "http://example.com/api", 0, None)]
    fn test_parse_http_methods(
        #[case] content: &str,
        #[case] expected_method: &str,
        #[case] expected_url: &str,
        #[case] expected_headers_count: usize,
        #[case] expected_body: Option<&str>,
    ) {
        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, expected_method);
        assert_eq!(requests[0].url, expected_url);
        assert_eq!(requests[0].headers.len(), expected_headers_count);
        assert_eq!(requests[0].body.as_deref(), expected_body);
    }

    #[test]
    fn test_parse_simple_get_with_header() {
        let content = "GET http://example.com/api\nAccept: application/json";
        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].url, "http://example.com/api");
        assert_eq!(requests[0].headers.get("Accept"), Some(&"application/json".to_string()));
    }

    #[rstest]
    #[case(
        r#"POST http://example.com/api
Content-Type: application/json

{"name": "test"}"#,
        "POST",
        r#"{"name": "test"}"#
    )]
    #[case(
        r#"POST http://example.com/api
Content-Type: application/json

{
  "name": "John",
  "email": "john@example.com"
}"#,
        "POST",
        r#"{
  "name": "John",
  "email": "john@example.com"
}"#
    )]
    #[case(
        r#"POST http://example.com/api
Content-Type: application/xml

<?xml version="1.0"?>
<request>
  <name>Test</name>
</request>"#,
        "POST",
        r#"<?xml version="1.0"?>
<request>
  <name>Test</name>
</request>"#
    )]
    fn test_parse_requests_with_body(
        #[case] content: &str,
        #[case] expected_method: &str,
        #[case] expected_body: &str,
    ) {
        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, expected_method);
        assert_eq!(requests[0].body.as_deref(), Some(expected_body));
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
        assert_eq!(requests[0].url, "http://example.com/api/1");
        assert_eq!(requests[1].method, "POST");
        assert_eq!(requests[1].url, "http://example.com/api/2");

        // Verify line numbers match actual positions
        let lines: Vec<&str> = content.lines().collect();
        let first_get_line = lines.iter().position(|l| l.trim().starts_with("GET")).unwrap();
        let first_post_line = lines.iter().position(|l| l.trim().starts_with("POST")).unwrap();
        assert_eq!(requests[0].line_number, first_get_line);
        assert_eq!(requests[1].line_number, first_post_line);
    }

    #[test]
    fn test_parse_multiple_headers() {
        let content = r#"GET http://example.com/api
Accept: application/json
Authorization: Bearer token123
User-Agent: Test/1.0
X-Custom-Header: custom-value"#;

        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].headers.len(), 4);
        assert_eq!(requests[0].headers.get("Accept"), Some(&"application/json".to_string()));
        assert_eq!(requests[0].headers.get("Authorization"), Some(&"Bearer token123".to_string()));
        assert_eq!(requests[0].headers.get("User-Agent"), Some(&"Test/1.0".to_string()));
        assert_eq!(requests[0].headers.get("X-Custom-Header"), Some(&"custom-value".to_string()));
    }

    #[rstest]
    #[case("# Just comments\n// More comments\n# No actual requests")]
    #[case("")]
    #[case("   \n   \n   ")]
    fn test_parse_empty_or_comment_only_files(#[case] content: &str) {
        let requests = parse_http_file(content);
        assert_eq!(requests.len(), 0);
    }

    #[test]
    fn test_parse_with_comments() {
        let content = r#"# This is a comment
// Another comment
GET http://example.com/api
Accept: application/json"#;

        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].url, "http://example.com/api");
    }

    #[test]
    fn test_parse_with_leading_empty_lines() {
        let content = r#"


GET http://example.com/api"#;

        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "GET");
    }

    #[test]
    fn test_parse_request_with_query_params() {
        let content = "GET http://example.com/api?page=1&limit=10&sort=desc";
        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].url, "http://example.com/api?page=1&limit=10&sort=desc");
    }

    #[test]
    fn test_parse_tracks_line_numbers() {
        let content = r#"// Comment line 0

GET http://example.com/api/1

###

POST http://example.com/api/2"#;

        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 2);
        // Line numbers are 0-indexed based on how parser stores them
        // Line 0: comment, Line 1: empty, Line 2: GET but stored as index
        let lines: Vec<&str> = content.lines().collect();
        let first_get_line = lines.iter().position(|l| l.trim().starts_with("GET")).unwrap();
        let first_post_line = lines.iter().position(|l| l.trim().starts_with("POST")).unwrap();

        assert_eq!(requests[0].line_number, first_get_line);
        assert_eq!(requests[1].line_number, first_post_line);
    }

    #[rstest]
    #[case(
        r#"POST http://example.com/api
Content-Type: application/json

"#,
        None
    )]
    #[case(
        r#"POST http://example.com/api
Content-Type: application/json"#,
        None
    )]
    fn test_parse_empty_body_scenarios(
        #[case] content: &str,
        #[case] expected_body: Option<&str>,
    ) {
        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].body.as_deref(), expected_body);
    }

    #[test]
    fn test_parse_three_consecutive_requests() {
        let content = r#"GET http://example.com/api/1
###
POST http://example.com/api/2
###
DELETE http://example.com/api/3"#;

        let requests = parse_http_file(content);

        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[1].method, "POST");
        assert_eq!(requests[2].method, "DELETE");

        // Verify line numbers
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(requests[0].line_number, lines.iter().position(|l| l.trim().starts_with("GET")).unwrap());
        assert_eq!(requests[1].line_number, lines.iter().position(|l| l.trim().starts_with("POST")).unwrap());
        assert_eq!(requests[2].line_number, lines.iter().position(|l| l.trim().starts_with("DELETE")).unwrap());
    }
}
