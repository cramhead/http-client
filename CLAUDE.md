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
- `language_server_command()`: Currently returns "Not implemented" - this is where LSP server setup would go if needed

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

## Current State

**Phase 1 Complete: Syntax Highlighting**
- ✅ Basic structure is in place
- ✅ HTTP file language support configured
- ✅ Tree-sitter grammar configured (rest-nvim/tree-sitter-http @ e061995)
- ✅ Syntax highlighting working for .http files
- ❌ Core request execution logic not yet implemented (commented out in src/lib.rs)
- ❌ Tests are commented out

**What Works:**
- Syntax highlighting for HTTP methods, URLs, headers, bodies
- Support for .http file extension
- Comment highlighting
- Variable highlighting (@variable syntax)

## Development Workflow

1. Make changes to Rust code in `src/`
2. Build with appropriate cargo command (see above)
3. Test using the `test/test.http` file
4. The extension.wasm file is the compiled output that Zed loads
