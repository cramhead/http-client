use crate::{executor, parser};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use std::fs::OpenOptions;
use std::io::Write;

pub struct HttpLspServer {
    client: Client,
    document_map: Arc<Mutex<HashMap<Url, String>>>,
}

impl HttpLspServer {
    pub fn new(client: Client) -> Self {
        // Log to file for debugging
        let _ = Self::log_to_file("HTTP LSP Server created");

        HttpLspServer {
            client,
            document_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn log_to_file(msg: &str) {
        let log_path = "/tmp/http-lsp.log";
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
        {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(file, "[{}] {}", timestamp, msg);
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for HttpLspServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        Self::log_to_file("Initialize called");

        // Log initialization
        self.client
            .log_message(MessageType::INFO, "HTTP LSP Server initializing...")
            .await;

        self.client
            .log_message(MessageType::INFO, format!("Client capabilities: {:?}", params.capabilities))
            .await;

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "HTTP LSP".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["http.sendRequest".to_string()],
                    ..Default::default()
                }),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "HTTP LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        self.document_map.lock().await.insert(uri.clone(), text);

        self.client
            .log_message(MessageType::INFO, format!("Opened document: {}", uri))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(change) = params.content_changes.first() {
            self.document_map
                .lock()
                .await
                .insert(uri.clone(), change.text.clone());
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.document_map.lock().await.remove(&params.text_document.uri);
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let line = params.range.start.line as usize;

        Self::log_to_file(&format!("Code action requested for: {} at line {}", uri, line));

        let document_map = self.document_map.lock().await;
        let content = match document_map.get(&uri) {
            Some(content) => content.clone(),
            None => {
                Self::log_to_file("Document not found in map");
                return Ok(None);
            }
        };
        drop(document_map);

        let requests = parser::parse_http_file(&content);

        Self::log_to_file(&format!("Found {} HTTP requests", requests.len()));
        for req in &requests {
            Self::log_to_file(&format!("  - {} {} at line {}", req.method, req.url, req.line_number));
        }

        // Find request at the current line - find the closest one before or at this line
        let request = requests.iter()
            .filter(|r| r.line_number <= line)
            .max_by_key(|r| r.line_number);

        if let Some(request) = request {
            Self::log_to_file(&format!("Found request {} at line {}", request.method, request.line_number));

            let action = CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("▶ Send {} Request", request.method),
                kind: Some(CodeActionKind::EMPTY),
                diagnostics: None,
                edit: None,
                command: Some(Command {
                    title: format!("Send {} Request", request.method),
                    command: "http.sendRequest".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(&uri.to_string()).unwrap(),
                        serde_json::to_value(request.line_number).unwrap(),
                    ]),
                }),
                is_preferred: Some(true),
                disabled: None,
                data: None,
            });

            return Ok(Some(vec![action]));
        }

        Ok(None)
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri;

        Self::log_to_file(&format!("Code lens requested for: {}", uri));

        self.client
            .log_message(MessageType::INFO, format!("Code lens requested for: {}", uri))
            .await;

        let document_map = self.document_map.lock().await;
        let content = match document_map.get(&uri) {
            Some(content) => content.clone(),
            None => {
                self.client
                    .log_message(MessageType::WARNING, format!("Document not found in map: {}", uri))
                    .await;
                return Ok(None);
            }
        };
        drop(document_map);

        let requests = parser::parse_http_file(&content);

        Self::log_to_file(&format!("Found {} HTTP requests", requests.len()));
        for req in &requests {
            Self::log_to_file(&format!("  - {} {} at line {}", req.method, req.url, req.line_number));
        }

        self.client
            .log_message(MessageType::INFO, format!("Found {} HTTP requests", requests.len()))
            .await;

        let mut lenses = Vec::new();

        for request in requests {
            Self::log_to_file(&format!("Creating lens for {} at line {}", request.method, request.line_number));

            // Create a code lens above the request line
            let range = Range {
                start: Position {
                    line: request.line_number as u32,
                    character: 0,
                },
                end: Position {
                    line: request.line_number as u32,
                    character: 0,
                },
            };

            let lens = CodeLens {
                range,
                command: Some(Command {
                    title: format!("▶ Send {} Request", request.method),
                    command: "http.sendRequest".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(&uri.to_string()).unwrap(),
                        serde_json::to_value(request.line_number).unwrap(),
                    ]),
                }),
                data: None,
            };

            lenses.push(lens);
        }

        Self::log_to_file(&format!("Returning {} code lenses", lenses.len()));

        Ok(Some(lenses))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        if params.command == "http.sendRequest" {
            // Extract arguments
            let args = params.arguments;
            if args.len() >= 2 {
                let uri_str = args[0].as_str().unwrap_or("");
                let line_number: usize = args[1].as_u64().unwrap_or(0) as usize;

                let uri = Url::parse(uri_str).unwrap();

                let document_map = self.document_map.lock().await;
                let content = match document_map.get(&uri) {
                    Some(content) => content.clone(),
                    None => {
                        self.client
                            .log_message(MessageType::ERROR, "Document not found")
                            .await;
                        return Ok(None);
                    }
                };
                drop(document_map);

                // Parse requests and find the one at the specified line
                let requests = parser::parse_http_file(&content);
                let request = requests.iter()
                    .filter(|r| r.line_number == line_number)
                    .next();

                if let Some(request) = request {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!("Executing {} request to {}", request.method, request.url),
                        )
                        .await;

                    // Execute the request
                    match executor::execute_request(request).await {
                        Ok(response) => {
                            // Create a formatted response document
                            let response_content = self.format_response_output(request, &response);

                            // Get the workspace root from the URI
                            let workspace_root = if let Some(segments) = uri.path_segments() {
                                let path = uri.path();
                                // Find the project root by looking for common indicators
                                if let Some(pos) = path.rfind("/test/") {
                                    &path[..pos]
                                } else if let Some(pos) = path.rfind("/src/") {
                                    &path[..pos]
                                } else {
                                    // Fallback: use parent directory
                                    std::path::Path::new(path)
                                        .parent()
                                        .and_then(|p| p.to_str())
                                        .unwrap_or("/tmp")
                                }
                            } else {
                                "/tmp"
                            };

                            let output_file = format!("{}/http-responses.http", workspace_root);
                            Self::log_to_file(&format!("Writing response to output file: {}", output_file));

                            // Prepare content with separator
                            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                            let separator = "=".repeat(80);
                            let header = format!("{}\n[{}]\n{}\n", separator, timestamp, separator);
                            let full_content = format!("{}{}\n\n", header, response_content);

                            // Append to the output file
                            use std::io::Write;
                            match OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(&output_file)
                            {
                                Ok(mut file) => {
                                    if let Err(e) = file.write_all(full_content.as_bytes()) {
                                        Self::log_to_file(&format!("Failed to write to output file: {}", e));
                                    } else {
                                        Self::log_to_file("Response written successfully");

                                        // Show success message with file location
                                        self.client
                                            .show_message(
                                                MessageType::INFO,
                                                format!("✓ {} - Response appended to http-responses.http", response.summary()),
                                            )
                                            .await;
                                    }
                                }
                                Err(e) => {
                                    Self::log_to_file(&format!("Failed to open output file: {}", e));

                                    self.client
                                        .show_message(
                                            MessageType::ERROR,
                                            format!("Failed to write response: {}", e),
                                        )
                                        .await;
                                }
                            }

                            return Ok(Some(serde_json::to_value(response.summary()).unwrap()));
                        }
                        Err(e) => {
                            self.client
                                .show_message(MessageType::ERROR, format!("Request failed: {}", e))
                                .await;
                        }
                    }
                }
            }
        }

        Ok(None)
    }
}

impl HttpLspServer {
    fn format_response_output(&self, request: &parser::HttpRequest, response: &executor::HttpResponse) -> String {
        let mut output = String::new();

        // Request section
        output.push_str("### REQUEST ###\n");
        output.push_str(&format!("{} {}\n", request.method, request.url));

        if !request.headers.is_empty() {
            output.push('\n');
            for (name, value) in &request.headers {
                output.push_str(&format!("{}: {}\n", name, value));
            }
        }

        if let Some(body) = &request.body {
            output.push_str("\n");
            output.push_str(body);
            output.push('\n');
        }

        output.push_str("\n");
        output.push_str("### RESPONSE ###\n");

        // Response status line
        output.push_str(&format!("HTTP/1.1 {} {} ({}ms)\n",
            response.status,
            response.status_text,
            response.duration_ms));

        // Response headers
        output.push('\n');
        for (name, value) in &response.headers {
            output.push_str(&format!("{}: {}\n", name, value));
        }

        // Response body
        output.push_str("\n");

        // Try to pretty-print JSON
        if let Some(content_type) = response.headers.get("content-type") {
            if content_type.contains("application/json") {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response.body) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&json_value) {
                        output.push_str(&pretty);
                        output.push('\n');
                        return output;
                    }
                }
            }
        }

        // Default: just add the body as-is
        output.push_str(&response.body);
        output.push('\n');

        output
    }
}
