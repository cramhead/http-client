# HTTP Client for Zed

A Zed editor extension that enables sending HTTP requests to APIs directly from the editor. Test and debug APIs without leaving your development environment.

[![License](https://img.shields.io/github/license/cramhead/http-client)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.0.3-blue.svg)](extension.toml)

## Features

- 🚀 **Send HTTP requests** directly from `.http` files in Zed
- 🎨 **Syntax highlighting** for HTTP request files (provided by the [HTTP](https://github.com/zed-industries/zed/tree/main/extensions/http) plugin)
- 📝 **Support for all HTTP methods** (GET, POST, PUT, DELETE, PATCH, etc.)
- 🔧 **Headers and request bodies** with JSON, XML, and other formats
- 📊 **Response viewer** with formatted output
- 💻 **Cross-platform** - Automatic binary downloads for macOS, Linux, and Windows
- ⚡ **Fast and lightweight** - Built with Rust and WebAssembly

## Installation

### From Zed Extensions

1. Open Zed editor
2. Press `cmd+shift+p` (macOS) or `ctrl+shift+p` (Linux/Windows)
3. Type "extensions" and select "zed: extensions"
4. Search for "Http Client"
5. Click "Install"

### Manual Installation

Clone this repository and build from source:

```bash
git clone https://github.com/cramhead/http-client.git
cd http-client
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/http_client.wasm extension.wasm
```

## Usage

### Creating an HTTP Request File

Create a file with the `.http` extension and write your requests:

```http
# Simple GET request
GET https://api.github.com/users/octocat
Accept: application/json

###

# POST request with JSON body
POST https://httpbin.org/post
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}

###

# Request with headers
GET https://api.example.com/data
Authorization: Bearer your-token-here
Accept: application/json
```

### Request Syntax

- **Request Line**: `METHOD URL`
- **Headers**: `Header-Name: value` (one per line)
- **Body**: Leave a blank line after headers, then add your request body
- **Separator**: Use `###` to separate multiple requests in one file

### Executing Requests

1. Open a `.http` file
2. Use code actions (code lenses) to execute requests
3. View results in the `http-responses.http` file

## How It Works

### Architecture

The extension consists of two main components:

#### 1. Zed Extension (WebAssembly)
- **Location**: `src/lib.rs`
- **Purpose**: Integrates with Zed's extension system
- **Responsibilities**:
  - Registers `.http` file support
  - Manages LSP server lifecycle
  - Handles platform detection and binary distribution
  - Downloads and caches the appropriate LSP server binary

#### 2. LSP Server (Native Binary)
- **Location**: `lsp/`
- **Purpose**: Language Server Protocol implementation
- **Responsibilities**:
  - Parses `.http` files
  - Executes HTTP requests
  - Formats responses
  - Provides code lenses for request execution

### Binary Distribution

The extension automatically downloads the correct LSP server binary for your platform:

1. **Platform Detection**: Detects your OS and architecture
2. **Cache Check**: Looks for cached binary
3. **Download**: If not cached, downloads from GitHub Releases
4. **Execution**: Makes binary executable and starts the LSP server

Supported platforms:
- macOS (Intel x86_64 and Apple Silicon aarch64)
- Linux (x86_64 and aarch64)
- Windows (x86_64)

## Development

### Prerequisites

- Rust toolchain (install from [rustup.rs](https://rustup.rs))
- `wasm32-wasip1` target: `rustup target add wasm32-wasip1`
- Zed editor

### Project Structure

```
http-client/
├── src/               # Extension code (WebAssembly)
│   └── lib.rs        # Main extension entry point
├── lsp/              # LSP server (native binary)
│   ├── src/
│   │   ├── main.rs      # LSP server entry point
│   │   ├── parser.rs    # HTTP request parser
│   │   ├── executor.rs  # Request executor
│   │   └── lsp_server.rs # LSP implementation
│   └── Cargo.toml
├── languages/http/    # Language configuration
│   ├── config.toml   # Language settings
│   └── highlights.scm # Syntax highlighting
├── .github/workflows/ # CI/CD
│   └── release.yml   # Release automation
├── extension.toml    # Extension metadata
└── CLAUDE.md        # Development guidelines
```

### Building

**Build the extension:**
```bash
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/http_client.wasm extension.wasm
```

**Build the LSP server:**
```bash
cd lsp
cargo build --release
```

For development, create a `bin/` directory in your workspace and copy the LSP binary there. The extension will use this instead of downloading:

```bash
mkdir -p bin
cp lsp/target/release/http-lsp bin/
```

### Testing

Run the test suite:

```bash
# Run all tests (51 tests across both crates)
cargo test --workspace

# Run extension tests only
cargo test -p http-client

# Run LSP tests only
cargo test -p http-lsp
```

Tests use `rstest` for parametrized testing and cover:
- HTTP request parsing (all methods, headers, bodies)
- Response formatting and display
- Edge cases (comments, empty files, multiple requests)

## Contributing

Contributions are welcome! We appreciate your help in making this extension better.

### How to Contribute

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes** and add tests
4. **Run tests**: `cargo test --workspace`
5. **Commit your changes**: `git commit -m 'Add amazing feature'`
6. **Push to your fork**: `git push origin feature/amazing-feature`
7. **Open a Pull Request**

### Contribution Guidelines

- Write clear, concise commit messages
- Add tests for new features
- Update documentation as needed
- Follow Rust best practices and idioms
- Run `cargo fmt` and `cargo clippy` before committing

### Areas for Contribution

We're particularly interested in contributions for:

- 🌐 Additional HTTP features (OAuth, certificates, proxies)
- 📝 Improved syntax highlighting
- 🎨 Better response formatting
- 🐛 Bug fixes and performance improvements
- 📚 Documentation improvements
- 🧪 Additional tests and test coverage

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive experience for everyone. We pledge to make participation in this project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Expected Behavior

- **Be respectful**: Treat everyone with respect and kindness
- **Be constructive**: Provide helpful feedback and suggestions
- **Be collaborative**: Work together to solve problems
- **Be patient**: Remember that everyone has different skill levels
- **Be open-minded**: Consider different perspectives and approaches

### Unacceptable Behavior

- Harassment, discrimination, or offensive comments
- Personal attacks or insults
- Trolling or inflammatory comments
- Publishing private information without consent
- Any conduct that would be inappropriate in a professional setting

### Reporting

If you experience or witness unacceptable behavior, please report it by opening an issue or contacting the maintainers directly. All reports will be handled with discretion and confidentiality.

### Enforcement

Violations of the Code of Conduct may result in:
1. A warning
2. Temporary ban from the project
3. Permanent ban from the project

We reserve the right to remove comments, commits, code, issues, and other contributions that violate this Code of Conduct.

## Feedback

We value your feedback! Here's how you can share it:

- 🐛 **Bug Reports**: [Open an issue](https://github.com/cramhead/http-client/issues/new) with details about the problem
- 💡 **Feature Requests**: [Start a discussion](https://github.com/cramhead/http-client/discussions) about your idea
- ❓ **Questions**: Use [GitHub Discussions](https://github.com/cramhead/http-client/discussions) for questions
- ⭐ **Show Support**: Star the repository if you find it useful!

## Roadmap

Future plans for the extension:

- [ ] Environment variables and variable substitution
- [ ] Request history and favorites
- [ ] GraphQL support
- [ ] WebSocket support
- [ ] Import/export collections (Postman, Insomnia)
- [ ] Authentication flows (OAuth, JWT)
- [ ] Request chaining and scripting
- [ ] Performance improvements

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0) - see the [LICENSE](LICENSE) file for details.

### What this means:

- ✅ You can use, modify, and distribute this software
- ✅ You can use it for commercial purposes
- ⚠️ If you modify and distribute it, you must share your changes under the same license
- ⚠️ If you use it in a network service, you must make the source code available

## Acknowledgments

- Built with [zed_extension_api](https://docs.rs/zed_extension_api/)
- Tree-sitter grammar from [rest-nvim/tree-sitter-http](https://github.com/rest-nvim/tree-sitter-http)
- Inspired by REST Client and HTTP Client tools

## Author

**Marc d'Entremont** ([@cramhead](https://github.com/cramhead))

---

**Note**: This extension is in active development. If you encounter any issues or have suggestions, please don't hesitate to reach out!
