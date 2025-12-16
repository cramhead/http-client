use zed_extension_api as zed;

pub struct HttpClient;

impl zed::Extension for HttpClient {
    fn new() -> Self {
        HttpClient
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed_extension_api::LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> zed_extension_api::Result<zed_extension_api::Command> {
        // Try to find the LSP binary in order of preference:
        // 1. In the workspace (for development)
        // 2. Bundled with the extension (for distribution)

        let worktree_root = worktree.root_path();

        // Development path: in the project's bin/ directory
        let dev_path = format!("{}/bin/http-lsp", worktree_root);

        // Check if development binary exists
        let lsp_path = if std::fs::metadata(&dev_path).is_ok() {
            dev_path
        } else {
            // TODO: For distribution, the binary should be bundled with the extension
            // and accessed via a different mechanism. For now, fall back to dev path.
            // In a distributed extension, you would use:
            // - Download binary on first use to a cache directory
            // - Or bundle in extension and use extension_dir path
            dev_path
        };

        Ok(zed::Command {
            command: lsp_path,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(HttpClient);

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
