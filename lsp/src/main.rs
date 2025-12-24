mod executor;
mod lsp_server;
mod parser;

use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(lsp_server::HttpLspServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}
