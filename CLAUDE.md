# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Zed editor extension called "Http Client" that enables sending HTTP requests to APIs directly from the editor. The extension supports .http files with a syntax similar to REST Client/HTTP Client formats.

## Build Commands

Build the extension WASM binary:
```bash
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/http_client.wasm extension.wasm
```

For development builds (faster):
```bash
cargo build --target wasm32-wasip1
cp target/wasm32-wasip1/debug/http_client.wasm extension.wasm
```

## Testing

The project is set up as a Cargo workspace with two members:
- Root crate: The Zed extension
- `lsp/`: The LSP server

Run all tests (52 tests across both crates):
```bash
cargo test --workspace
# Or use the alias:
cargo t
```

Run tests for specific crate:
```bash
cargo test -p http-client  # Extension tests (3 tests)
cargo test -p http-lsp     # LSP tests (49 tests)
```

Tests use `rstest` for parametrized/table-based testing.

## Architecture

### Extension Structure

The project follows the Zed extension architecture:

- `src/lib.rs`: Main extension entry point implementing the `zed::Extension` trait
- `extension.toml`: Extension metadata and grammar configuration
- `languages/http/config.toml`: Language configuration for .http files
- `languages/http/highlights.scm`: Tree-sitter syntax highlighting queries
- `test/test.http`: Sample HTTP request file demonstrating syntax
- Note: Grammar source is fetched by Zed from the repository specified in extension.toml

### Extension Entry Point

The `HttpClient` struct implements `zed::Extension` with:
- `new()`: Initialize extension state
- `language_server_command()`: Handles LSP server binary distribution and lifecycle
  - Detects platform (OS and architecture)
  - Downloads platform-specific binary from GitHub Releases on first use
  - Caches binary for subsequent launches
  - Falls back to development binary in `bin/http-lsp` for local development

The extension is registered using `zed::register_extension!(HttpClient)`.

### HTTP File Format

Files with `.http` extension contain HTTP requests separated by `###` delimiters. Each request includes:
- HTTP method and URL
- Optional headers (key: value format)
- Optional request body (JSON, XML, etc.)

Example structure from test/test.http:
```
GET http://example.com/api/data
Accept: application/json

###

POST http://example.com/api/users
Content-Type: application/json

{
  "name": "John Doe"
}
```

## Key Implementation Notes

- Uses Zed Extension API v0.7.0 (`zed_extension_api` crate)
- Built as a WebAssembly module (`cdylib` crate type)
- Target: `wasm32-wasip1`
- Language definition uses tree-sitter grammar for syntax highlighting (from rest-nvim/tree-sitter-http)
- Grammar is fetched by Zed based on `[grammars.http]` section in extension.toml
- Syntax highlights defined in `languages/http/highlights.scm`
- Bracket pairs defined in language config: quotes, braces, angle brackets

## Binary Distribution

The extension uses a sophisticated binary distribution system:

### Platform Support
- macOS: Intel (x86_64) and Apple Silicon (aarch64)
- Linux: x86_64 and aarch64
- Windows: x86_64

### Distribution Flow
1. **Platform Detection**: Uses `zed::current_platform()` to detect OS and architecture
2. **Cache Check**: Looks for binary in `bin/` directory
3. **Download**: If not cached, fetches from GitHub Releases using `zed::latest_github_release()`
4. **Installation**: Downloads binary, makes it executable (Unix), and caches it
5. **Execution**: Returns cached binary path to Zed

### Development Mode
For local development, the extension prioritizes `{workspace}/bin/http-lsp` over downloading. This allows developers to test changes without triggering downloads.

### CI/CD
GitHub Actions workflow (`.github/workflows/release.yml`) automatically:
- Builds binaries for all 5 platforms when a version tag is pushed
- Uses cross-compilation for platform-specific builds
- Uploads binaries to GitHub Releases with standardized names:
  - `http-lsp-macos-x86_64`
  - `http-lsp-macos-aarch64`
  - `http-lsp-linux-x86_64`
  - `http-lsp-linux-aarch64`
  - `http-lsp-windows-x86_64.exe`

## Current State

**Phase 1 Complete: Full Extension Implementation**
- ✅ Basic structure is in place
- ✅ HTTP file language support configured
- ✅ Tree-sitter grammar configured (rest-nvim/tree-sitter-http @ e061995)
- ✅ Syntax highlighting working for .http files
- ✅ Comprehensive test suite with 52 tests covering:
  - HTTP request parsing (all methods, headers, bodies)
  - Response formatting and display
  - LSP server response output formatting
  - Edge cases (comments, empty files, multiple requests)
  - Extension lifecycle and structure
- ✅ LSP server implementation with request execution
- ✅ Cross-platform binary distribution system
- ✅ Automatic LSP server download and caching
- ✅ CI/CD pipeline for building release binaries

**What Works:**
- Syntax highlighting for HTTP methods, URLs, headers, bodies
- Support for .http file extension
- Comment highlighting
- Variable highlighting (@variable syntax)
- LSP server with code lenses and request execution
- HTTP request parsing and execution
- Response formatting and display
- Automatic platform detection (macOS x64/ARM, Linux x64/ARM, Windows x64)
- Binary download from GitHub Releases
- Binary caching for offline use
- Development mode with local binaries

## Development Workflow

1. Make changes to Rust code in `src/`
2. Build with appropriate cargo command (see above)
3. Test using the `test/test.http` file
4. The extension.wasm file is the compiled output that Zed loads
