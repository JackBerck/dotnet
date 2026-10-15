//! Nginx Virtual Host Site Generator, validation, and DNS routing.

use crate::error::{CoreError, Result};
use crate::models::ProjectInfo;
use crate::persistence::AppPaths;
use crate::services::cert_manager::LocalCertManager;
use crate::services::hosts_manager::HostsFileManager;
use crate::services::nginx_manager::NginxManager;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static SAFE_FILE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^a-z0-9\-]+").expect("valid regex"));

const RESTRICTED_DOMAINS: &[&str] = &[
    "google.com",
    "github.com",
    "facebook.com",
    "microsoft.com",
    "apple.com",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NginxSiteGeneratorResult {
    pub success: bool,
    pub config_path: PathBuf,
    pub message: String,
    pub ssl_enabled: bool,
}

pub struct NginxSiteGenerator {
    sites_dir: PathBuf,
    nginx_dir: PathBuf,
    hosts_file: PathBuf,
}

impl Default for NginxSiteGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl NginxSiteGenerator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sites_dir: AppPaths::get_path("nginx").join("sites"),
            nginx_dir: NginxManager::default_nginx_dir(),
            hosts_file: HostsFileManager::default_hosts_path(),
        }
    }

    #[must_use]
    pub fn with_custom_paths(sites_dir: PathBuf, nginx_dir: PathBuf, hosts_file: PathBuf) -> Self {
        Self {
            sites_dir,
            nginx_dir,
            hosts_file,
        }
    }

    #[must_use]
    pub fn sites_dir(&self) -> &Path {
        &self.sites_dir
    }

    /// Build Nginx vhost configuration string for a given project.
    #[must_use]
    pub fn build_site_config(
        project: &ProjectInfo,
        fastcgi_port: u16,
        enable_ssl: bool,
        cert_path: Option<&Path>,
        key_path: Option<&Path>,
    ) -> String {
        let host = project.effective_host();
        let full_root = project
            .resolved_public_path()
            .to_string_lossy()
            .replace('\\', "/");

        let mut out = String::with_capacity(1024);

        let ssl_creds = if enable_ssl {
            match (cert_path, key_path) {
                (Some(c), Some(k)) => Some((c, k)),
                _ => None,
            }
        } else {
            None
        };

        if let Some((cert, key)) = ssl_creds {
            let fwd_cert = cert.to_string_lossy().replace('\\', "/");
            let fwd_key = key.to_string_lossy().replace('\\', "/");

            // HTTP -> HTTPS 301 redirect
            out.push_str("server {\n");
            out.push_str("    listen 80;\n");
            out.push_str(&format!("    server_name {host};\n"));
            out.push_str("    return 301 https://$host$request_uri;\n");
            out.push_str("}\n\n");

            // HTTPS block
            out.push_str("server {\n");
            out.push_str("    listen 443 ssl;\n");
            out.push_str(&format!("    server_name {host};\n"));
            out.push_str(&format!("    ssl_certificate \"{fwd_cert}\";\n"));
            out.push_str(&format!("    ssl_certificate_key \"{fwd_key}\";\n"));
            out.push_str("    ssl_protocols TLSv1.2 TLSv1.3;\n");
            out.push_str("    ssl_ciphers HIGH:!aNULL:!MD5;\n\n");
        } else {
            // Plain HTTP block
            out.push_str("server {\n");
            out.push_str("    listen 80;\n");
            out.push_str(&format!("    server_name {host};\n\n"));
        }

        if project.is_node_based() {
            let port = project.dev_port.unwrap_or_else(|| {
                if project.framework.eq_ignore_ascii_case("vite") {
                    5173
                } else {
                    3000
                }
            });

            out.push_str("    location / {\n");
            out.push_str(&format!("        proxy_pass http://127.0.0.1:{port};\n"));
            out.push_str("        proxy_http_version 1.1;\n");
            out.push_str("        proxy_set_header Upgrade $http_upgrade;\n");
            out.push_str("        proxy_set_header Connection \"upgrade\";\n");
            out.push_str("        proxy_set_header Host $host;\n");
            out.push_str("        proxy_cache_bypass $http_upgrade;\n");
            out.push_str("    }\n");
        } else if project.is_static() {
            out.push_str(&format!("    root \"{full_root}\";\n"));
            out.push_str("    index index.html index.htm;\n\n");
            out.push_str("    location / {\n");
            out.push_str("        try_files $uri $uri/ =404;\n");
            out.push_str("    }\n");
        } else {
            // PHP / Laravel / Custom FastCGI
            out.push_str(&format!("    root \"{full_root}\";\n"));
            out.push_str("    index index.php index.html index.htm;\n\n");
            out.push_str("    location / {\n");
            out.push_str("        try_files $uri $uri/ /index.php?$query_string;\n");
            out.push_str("    }\n\n");
            out.push_str("    location ~ \\.php$ {\n");
            out.push_str(&format!("        fastcgi_pass 127.0.0.1:{fastcgi_port};\n"));
            out.push_str("        fastcgi_index index.php;\n");
            out.push_str(
                "        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;\n",
            );
            out.push_str("        include fastcgi_params;\n");
            out.push_str("    }\n");
        }

        out.push_str("}\n");
        out
    }

    /// Ensure main nginx.conf includes this sites directory.
    pub fn ensure_nginx_includes_sites_dir(&self) -> Result<()> {
        let nginx_conf = self.nginx_dir.join("conf").join("nginx.conf");
        if !nginx_conf.is_file() {
            return Ok(());
        }

        let content = fs::read_to_string(&nginx_conf)?;
        let sites_pattern = format!(
            "include \"{}/*.conf\";",
            self.sites_dir.to_string_lossy().replace('\\', "/")
        );

        if content.contains(&sites_pattern)
            || content.contains("nginx/sites/*.conf")
            || content.contains("conf/sites/*.conf")
        {
            return Ok(());
        }

        if let Some(last_brace) = content.rfind('}') {
            let mut updated = content[..last_brace].to_string();
            updated.push_str("\n    # >>> Dotnet Managed Sites >>>\n");
            updated.push_str(&format!("    {sites_pattern}\n"));
            updated.push_str("    # <<< Dotnet Managed Sites <<<\n");
            updated.push_str(&content[last_brace..]);

            fs::write(&nginx_conf, updated)?;
            tracing::info!("NginxSiteGenerator: Injected sites include directive into nginx.conf");
        }

        Ok(())
    }

    /// Check whether a hostname is restricted (prevents DNS hijacking of major services).
    #[must_use]
    pub fn is_restricted_domain(host: &str) -> bool {
        let lower = host.to_lowercase();
        RESTRICTED_DOMAINS
            .iter()
            .any(|&d| lower == d || lower.ends_with(&format!(".{d}")))
    }

    /// Generate or update an Nginx virtual host site with rollback on validation failure.
    pub async fn create_or_update_site(
        &self,
        project: &ProjectInfo,
        enable_ssl: bool,
        register_host: bool,
    ) -> Result<NginxSiteGeneratorResult> {
        let host = project.effective_host();
        if host.is_empty() || host == "localhost" {
            return Err(CoreError::Config(
                "Invalid or empty host domain.".to_string(),
            ));
        }

        if !HostsFileManager::is_valid_hostname(host) {
            return Err(CoreError::Config(format!(
                "Invalid RFC 1123 hostname: '{host}'"
            )));
        }

        if Self::is_restricted_domain(host) {
            return Err(CoreError::Config(format!(
                "Host domain '{host}' is restricted to protect public internet services."
            )));
        }

        fs::create_dir_all(&self.sites_dir)?;
        let _ = self.ensure_nginx_includes_sites_dir();

        let raw_name = if project.name.trim().is_empty() {
            "project"
        } else {
            project.name.trim()
        };
        let lower_name = raw_name.to_lowercase();
        let sanitized = SAFE_FILE_REGEX.replace_all(&lower_name, "-");
        let safe_name = sanitized.trim_matches('-');
        let file_stem = if safe_name.is_empty() {
            "project"
        } else {
            safe_name
        };
        let target_conf_path = self.sites_dir.join(format!("{file_stem}.conf"));

        // SSL Certificate generation
        let (cert_path, key_path) = if enable_ssl {
            let (c, k) = LocalCertManager::generate_domain_cert(host)?;
            (Some(c), Some(k))
        } else {
            (None, None)
        };

        let content = Self::build_site_config(
            project,
            9000,
            enable_ssl,
            cert_path.as_deref(),
            key_path.as_deref(),
        );

        let old_backup = if target_conf_path.exists() {
            Some(fs::read_to_string(&target_conf_path)?)
        } else {
            None
        };

        // Write candidate config
        fs::write(&target_conf_path, &content)?;

        // If nginx.exe exists in nginx_dir, validate syntax
        let nginx_exe = NginxManager::get_nginx_exe(&self.nginx_dir);
        if nginx_exe.is_file() {
            match NginxManager::validate_config(&self.nginx_dir).await {
                Ok((true, _)) => {
                    // Passed validation, proceed to reload
                    let _ = NginxManager::reload_nginx(&self.nginx_dir).await;
                }
                Ok((false, err_msg)) => {
                    // Rollback
                    if let Some(old) = old_backup {
                        let _ = fs::write(&target_conf_path, old);
                    } else {
                        let _ = fs::remove_file(&target_conf_path);
                    }
                    return Err(CoreError::Service(format!(
                        "Nginx configuration syntax validation failed. Rolled back.\nError: {err_msg}"
                    )));
                }
                Err(e) => {
                    tracing::warn!("Could not run nginx validation check: {e}");
                }
            }
        }

        // Register in hosts file if requested
        if register_host && self.hosts_file.exists() {
            if let Err(e) =
                HostsFileManager::add_or_update_managed_entry(&self.hosts_file, "127.0.0.1", host)
            {
                tracing::warn!("Failed to add entry to hosts file: {e}");
            }
        }

        tracing::info!(
            "NginxSiteGenerator: Successfully generated site for '{}' at {}",
            host,
            target_conf_path.display()
        );

        Ok(NginxSiteGeneratorResult {
            success: true,
            config_path: target_conf_path,
            message: format!(
                "Site '{}' is active and routed{}",
                host,
                if enable_ssl { " with HTTPS (SSL)" } else { "" }
            ),
            ssl_enabled: enable_ssl,
        })
    }

    /// Remove Nginx site configuration and unregister host entry.
    pub async fn remove_site(&self, project: &ProjectInfo) -> Result<bool> {
        let raw_name = if project.name.trim().is_empty() {
            "project"
        } else {
            project.name.trim()
        };
        let lower_name = raw_name.to_lowercase();
        let sanitized = SAFE_FILE_REGEX.replace_all(&lower_name, "-");
        let safe_name = sanitized.trim_matches('-');
        let file_stem = if safe_name.is_empty() {
            "project"
        } else {
            safe_name
        };
        let target_conf_path = self.sites_dir.join(format!("{file_stem}.conf"));

        let mut removed = false;
        if target_conf_path.exists() {
            fs::remove_file(&target_conf_path)?;
            removed = true;
        }

        if let Some(host) = &project.nginx_host {
            if self.hosts_file.exists() {
                let _ = HostsFileManager::remove_managed_entry(&self.hosts_file, host);
            }
        }

        let nginx_exe = NginxManager::get_nginx_exe(&self.nginx_dir);
        if nginx_exe.is_file() {
            let _ = NginxManager::reload_nginx(&self.nginx_dir).await;
        }

        tracing::info!(
            "NginxSiteGenerator: Removed site configuration for project '{}'",
            project.name
        );
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_build_site_config_laravel_ssl() {
        let mut proj = ProjectInfo::new("my-api", "D:/code/my-api");
        proj.framework = "laravel".into();
        proj.nginx_host = Some("my-api.test".into());
        proj.public_directory = "public".into();

        let cert = Path::new("D:/ssl/my-api.test.crt");
        let key = Path::new("D:/ssl/my-api.test.key");

        let conf = NginxSiteGenerator::build_site_config(&proj, 9000, true, Some(cert), Some(key));

        assert!(conf.contains("listen 80;"));
        assert!(conf.contains("return 301 https://$host$request_uri;"));
        assert!(conf.contains("listen 443 ssl;"));
        assert!(conf.contains("server_name my-api.test;"));
        assert!(conf.contains("fastcgi_pass 127.0.0.1:9000;"));
        assert!(conf.contains("ssl_certificate \"D:/ssl/my-api.test.crt\";"));
    }

    #[test]
    fn test_build_site_config_nextjs() {
        let mut proj = ProjectInfo::new("my-next", "D:/code/my-next");
        proj.framework = "nextjs".into();
        proj.nginx_host = Some("my-next.test".into());
        proj.dev_port = Some(3000);

        let conf = NginxSiteGenerator::build_site_config(&proj, 9000, false, None, None);

        assert!(conf.contains("listen 80;"));
        assert!(conf.contains("server_name my-next.test;"));
        assert!(conf.contains("proxy_pass http://127.0.0.1:3000;"));
        assert!(conf.contains("proxy_set_header Upgrade $http_upgrade;"));
    }

    #[test]
    fn test_build_site_config_static() {
        let mut proj = ProjectInfo::new("landing", "D:/code/landing");
        proj.framework = "static".into();
        proj.nginx_host = Some("landing.test".into());

        let conf = NginxSiteGenerator::build_site_config(&proj, 9000, false, None, None);

        assert!(conf.contains("try_files $uri $uri/ =404;"));
        assert!(conf.contains("index index.html index.htm;"));
    }

    #[test]
    fn test_is_restricted_domain() {
        assert!(NginxSiteGenerator::is_restricted_domain("google.com"));
        assert!(NginxSiteGenerator::is_restricted_domain("api.github.com"));
        assert!(NginxSiteGenerator::is_restricted_domain("facebook.com"));
        assert!(!NginxSiteGenerator::is_restricted_domain("myproject.test"));
        assert!(!NginxSiteGenerator::is_restricted_domain("dev.local"));
    }

    #[tokio::test]
    async fn test_site_generator_creation_and_removal() {
        let dir = tempdir().expect("tempdir");
        let sites_dir = dir.path().join("sites");
        let nginx_dir = dir.path().join("nginx");
        let hosts_file = dir.path().join("hosts");

        fs::create_dir_all(&sites_dir).expect("mkdir sites");
        fs::create_dir_all(&nginx_dir).expect("mkdir nginx");
        fs::write(&hosts_file, "127.0.0.1 localhost\r\n").expect("write hosts");

        let gen = NginxSiteGenerator::with_custom_paths(sites_dir, nginx_dir, hosts_file.clone());

        let mut proj = ProjectInfo::new("shop-app", "D:/projects/shop");
        proj.framework = "laravel".into();
        proj.nginx_host = Some("shop.test".into());

        let res = gen
            .create_or_update_site(&proj, false, true)
            .await
            .expect("create site");

        assert!(res.success);
        assert!(res.config_path.exists());

        // Check hosts file updated
        let hosts_content = fs::read_to_string(&hosts_file).expect("read hosts");
        assert!(hosts_content.contains("127.0.0.1  shop.test"));

        // Remove site
        let removed = gen.remove_site(&proj).await.expect("remove site");
        assert!(removed);
        assert!(!res.config_path.exists());

        let hosts_after = fs::read_to_string(&hosts_file).expect("read hosts after");
        assert!(!hosts_after.contains("shop.test"));
    }
}
