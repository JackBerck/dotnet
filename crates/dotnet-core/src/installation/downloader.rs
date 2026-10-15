//! Secure streaming tool downloader with allowed-host validation and SHA-256 verification.

use crate::error::{CoreError, Result};
use crate::persistence::AppPaths;
use reqwest::Url;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct ToolDownloader;

impl ToolDownloader {
    /// Validate that the URL is HTTPS and belongs to the allowed hosts list.
    pub fn validate_url(url_str: &str, allowed_hosts: &[String]) -> Result<Url> {
        let parsed = Url::parse(url_str)
            .map_err(|e| CoreError::Validation(format!("Invalid URL '{url_str}': {e}")))?;

        if parsed.scheme() != "https" {
            return Err(CoreError::Validation(format!(
                "Insecure scheme '{}'. Only HTTPS downloads are allowed.",
                parsed.scheme()
            )));
        }

        let host = parsed
            .host_str()
            .ok_or_else(|| CoreError::Validation("URL host is missing".to_string()))?;

        let is_allowed = allowed_hosts.iter().any(|allowed| {
            let allowed_clean = allowed.trim().trim_start_matches('.').to_lowercase();
            let host_clean = host.to_lowercase();
            host_clean == allowed_clean || host_clean.ends_with(&format!(".{allowed_clean}"))
        });

        if !is_allowed {
            return Err(CoreError::Validation(format!(
                "Host '{host}' is not in the allowed hosts list: {:?}",
                allowed_hosts
            )));
        }

        Ok(parsed)
    }

    /// Download tool archive or binary with streaming SHA-256 verification and caching.
    pub async fn download<F>(
        url: &str,
        expected_sha256: &str,
        allowed_hosts: &[String],
        mut progress_cb: F,
    ) -> Result<PathBuf>
    where
        F: FnMut(f64),
    {
        let validated_url = Self::validate_url(url, allowed_hosts)?;
        let expected_hash = expected_sha256.trim().to_lowercase();

        let downloads_dir = AppPaths::get_path("downloads");
        fs::create_dir_all(&downloads_dir)?;

        let final_path = downloads_dir.join(format!("{expected_hash}.bin"));

        // Check if verified cached file exists
        if final_path.exists() {
            if Self::verify_file_sha256(&final_path, &expected_hash)? {
                tracing::info!("Using cached verified download: {}", final_path.display());
                progress_cb(100.0);
                return Ok(final_path);
            }
            let _ = fs::remove_file(&final_path);
        }

        let part_filename = format!("{expected_hash}.part.{}", std::process::id());
        let part_path = downloads_dir.join(part_filename);

        tracing::info!("Starting download: {validated_url}");

        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .timeout(std::time::Duration::from_secs(600))
            .build()?;

        let mut response = client.get(validated_url).send().await?;
        if !response.status().is_success() {
            return Err(CoreError::Http(response.error_for_status().unwrap_err()));
        }

        let total_bytes = response.content_length();
        let mut file = File::create(&part_path)?;
        let mut hasher = Sha256::new();
        let mut downloaded_bytes: u64 = 0;

        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk)?;
            hasher.update(&chunk);

            downloaded_bytes += chunk.len() as u64;
            if let Some(total) = total_bytes {
                if total > 0 {
                    let pct = (downloaded_bytes as f64 / total as f64) * 100.0;
                    progress_cb(pct.min(99.9));
                }
            }
        }

        file.flush()?;
        drop(file);

        let computed_hash = format!("{:x}", hasher.finalize());

        if computed_hash != expected_hash {
            let _ = fs::remove_file(&part_path);
            return Err(CoreError::Validation(format!(
                "SHA-256 verification failed for {url}! Expected: {expected_hash}, Computed: {computed_hash}"
            )));
        }

        // Rename verified part file to final cache path
        if final_path.exists() {
            let _ = fs::remove_file(&final_path);
        }
        fs::rename(&part_path, &final_path)?;

        tracing::info!("Download and SHA-256 verified: {}", final_path.display());
        progress_cb(100.0);
        Ok(final_path)
    }

    /// Compute and verify SHA-256 of an existing file.
    pub fn verify_file_sha256<P: AsRef<Path>>(file_path: P, expected_sha256: &str) -> Result<bool> {
        let file_path = file_path.as_ref();
        if !file_path.exists() {
            return Ok(false);
        }

        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let computed = format!("{:x}", hasher.finalize());
        Ok(computed.eq_ignore_ascii_case(expected_sha256.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_allowed_hosts() {
        let allowed = vec!["windows.php.net".into(), "nodejs.org".into()];
        assert!(ToolDownloader::validate_url(
            "https://windows.php.net/downloads/php.zip",
            &allowed
        )
        .is_ok());
        assert!(
            ToolDownloader::validate_url("https://sub.nodejs.org/dist/node.zip", &allowed).is_ok()
        );
        assert!(ToolDownloader::validate_url("http://windows.php.net/php.zip", &allowed).is_err()); // non-https
        assert!(ToolDownloader::validate_url("https://malicious.com/php.zip", &allowed).is_err());
        // disallowed host
    }

    #[test]
    fn test_verify_file_sha256() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let valid_hash = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(ToolDownloader::verify_file_sha256(&file_path, valid_hash).unwrap());
        assert!(!ToolDownloader::verify_file_sha256(
            &file_path,
            "0000000000000000000000000000000000000000000000000000000000000000"
        )
        .unwrap());
    }
}
