use zed_extension_api as zed;
use zed_extension_api::{Architecture, LanguageServerId, Os, Result};
use std::fs;

pub struct HttpClient;

impl zed::Extension for HttpClient {
    fn new() -> Self {
        HttpClient
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let worktree_root = worktree.root_path();
        let dev_path = format!("{}/bin/http-lsp", worktree_root);

        // Development mode: Check if binary exists in workspace bin/
        if fs::metadata(&dev_path).is_ok() {
            return Ok(zed::Command {
                command: dev_path,
                args: vec![],
                env: Default::default(),
            });
        }

        // Production mode: Download platform-specific binary
        let platform = zed::current_platform();
        let binary_name = get_binary_name_for_platform(&platform);
        let binary_path = ensure_binary_cached(language_server_id, &binary_name)?;

        Ok(zed::Command {
            command: binary_path,
            args: vec![],
            env: Default::default(),
        })
    }
}

/// Get the binary name for the current platform
fn get_binary_name_for_platform(platform: &(Os, Architecture)) -> String {
    let os = match platform.0 {
        Os::Mac => "macos",
        Os::Linux => "linux",
        Os::Windows => "windows",
    };

    let arch = match platform.1 {
        Architecture::Aarch64 => "aarch64",
        Architecture::X8664 => "x86_64",
        Architecture::X86 => "x86",
    };

    let extension = if platform.0 == Os::Windows { ".exe" } else { "" };

    format!("http-lsp-{}-{}{}", os, arch, extension)
}

/// Ensure the binary is cached, downloading if necessary
fn ensure_binary_cached(language_server_id: &LanguageServerId, binary_name: &str) -> Result<String> {
    let binary_path = format!("bin/{}", binary_name);

    // Check if already cached
    if fs::metadata(&binary_path).is_ok() {
        return Ok(binary_path);
    }

    // Download the binary
    download_and_cache_binary(language_server_id, binary_name, &binary_path)
}

/// Download and cache the binary from GitHub releases
fn download_and_cache_binary(
    language_server_id: &LanguageServerId,
    binary_name: &str,
    binary_path: &str,
) -> Result<String> {
    // Show downloading status
    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::Downloading,
    );

    // Get latest release from GitHub
    // TODO: Update with actual GitHub repository once created
    let release = zed::latest_github_release(
        "your-username/http-client",
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )?;

    // Find the matching asset
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == binary_name)
        .ok_or_else(|| format!("No binary found for platform: {}", binary_name))?;

    // Download the binary
    zed::download_file(
        &asset.download_url,
        binary_path,
        zed::DownloadedFileType::Uncompressed,
    )
    .map_err(|e| format!("Failed to download binary: {}", e))?;

    // Make executable on Unix platforms
    let platform = zed::current_platform();
    if platform.0 != Os::Windows {
        zed::make_file_executable(binary_path)
            .map_err(|e| format!("Failed to make binary executable: {}", e))?;
    }

    // Show completion status
    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::None,
    );

    Ok(binary_path.to_string())
}

zed::register_extension!(HttpClient);

#[cfg(test)]
mod tests {
    use super::*;
    use zed::Extension;

    #[test]
    fn test_extension_creation() {
        // Extension should be created without panicking
        let _extension = HttpClient::new();
        /*
        If we reach here, creation succeeded
        */
    }

    #[test]
    fn test_extension_is_zero_sized() {
        // HttpClient is a unit struct with no fields, so it should be zero-sized
        assert_eq!(std::mem::size_of::<HttpClient>(), 0);
    }

    #[test]
    fn test_extension_struct_is_send_sync() {
        // Verify HttpClient implements Send and Sync (required for thread safety)
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HttpClient>();
    }
}
