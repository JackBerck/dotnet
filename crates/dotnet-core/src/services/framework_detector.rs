use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

static SAFE_HOST_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^a-z0-9\-]+").expect("valid regex"));

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrameworkDetectionResult {
    pub framework: String,
    pub suggested_name: String,
    pub dev_command: String,
    pub suggested_port: Option<u16>,
    pub public_directory: String,
    pub suggested_host: String,
    pub description: String,
}

impl Default for FrameworkDetectionResult {
    fn default() -> Self {
        Self {
            framework: "custom".to_string(),
            suggested_name: "project".to_string(),
            dev_command: String::new(),
            suggested_port: None,
            public_directory: ".".to_string(),
            suggested_host: "myproject.test".to_string(),
            description: "Custom Project".to_string(),
        }
    }
}

pub struct FrameworkDetector;

impl FrameworkDetector {
    #[must_use]
    pub fn detect(directory_path: &Path) -> FrameworkDetectionResult {
        let dir_name = directory_path
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("project");

        let lower_name = dir_name.to_lowercase();
        let sanitized = SAFE_HOST_REGEX.replace_all(&lower_name, "-");
        let trimmed_host = sanitized.trim_matches('-');

        let safe_host = if trimmed_host.is_empty() {
            "myproject.test".to_string()
        } else {
            format!("{trimmed_host}.test")
        };

        let mut result = FrameworkDetectionResult {
            framework: "custom".to_string(),
            suggested_name: dir_name.to_string(),
            dev_command: String::new(),
            suggested_port: None,
            public_directory: ".".to_string(),
            suggested_host: safe_host,
            description: "Custom Project".to_string(),
        };

        if !directory_path.is_dir() {
            return result;
        }

        let composer_file = directory_path.join("composer.json");
        let package_file = directory_path.join("package.json");
        let index_php = directory_path.join("index.php");
        let public_index_php = directory_path.join("public").join("index.php");
        let index_html = directory_path.join("index.html");

        // 1. PHP / Composer / Laravel
        if composer_file.is_file() {
            if let Ok(text) = fs::read_to_string(&composer_file) {
                let lower_text = text.to_lowercase();
                if lower_text.contains("\"laravel/framework\"") {
                    result.framework = "laravel".to_string();
                    result.dev_command = "php artisan serve".to_string();
                    result.suggested_port = Some(8000);
                    result.public_directory = "public".to_string();
                    result.description = "Laravel Web Application".to_string();
                    return result;
                }

                result.framework = "php".to_string();
                result.dev_command = "php -S 127.0.0.1:8000".to_string();
                result.suggested_port = Some(8000);
                result.public_directory = if directory_path.join("public").is_dir() {
                    "public".to_string()
                } else {
                    ".".to_string()
                };
                result.description = "PHP Project (Composer)".to_string();
                return result;
            }
        }

        // 2. Node / Next.js / Vite
        if package_file.is_file() {
            if let Ok(text) = fs::read_to_string(&package_file) {
                let lower_text = text.to_lowercase();
                if lower_text.contains("\"next\"") {
                    result.framework = "nextjs".to_string();
                    result.dev_command = "npm run dev".to_string();
                    result.suggested_port = Some(3000);
                    result.public_directory = ".".to_string();
                    result.description = "Next.js React Framework".to_string();
                    return result;
                }

                if lower_text.contains("\"vite\"") {
                    result.framework = "vite".to_string();
                    result.dev_command = "npm run dev".to_string();
                    result.suggested_port = Some(5173);
                    result.public_directory = ".".to_string();
                    result.description = "Vite Frontend Framework".to_string();
                    return result;
                }

                result.framework = "node".to_string();
                result.dev_command = "npm run dev".to_string();
                result.suggested_port = Some(3000);
                result.public_directory = ".".to_string();
                result.description = "Node.js Application".to_string();
                return result;
            }
        }

        // 3. Raw PHP
        if index_php.is_file() || public_index_php.is_file() {
            result.framework = "php".to_string();
            result.dev_command = "php -S 127.0.0.1:8000".to_string();
            result.suggested_port = Some(8000);
            result.public_directory = if directory_path.join("public").is_dir() {
                "public".to_string()
            } else {
                ".".to_string()
            };
            result.description = "PHP Application".to_string();
            return result;
        }

        // 4. Static Website
        if index_html.is_file() {
            result.framework = "static".to_string();
            result.dev_command = String::new();
            result.suggested_port = None;
            result.public_directory = ".".to_string();
            result.description = "Static Website".to_string();
            return result;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_detect_laravel() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("my-cool-laravel");
        fs::create_dir_all(&project_dir).expect("mkdir");
        fs::write(
            project_dir.join("composer.json"),
            r#"{"require": {"laravel/framework": "^11.0"}}"#,
        )
        .expect("write");

        let res = FrameworkDetector::detect(&project_dir);
        assert_eq!(res.framework, "laravel");
        assert_eq!(res.suggested_port, Some(8000));
        assert_eq!(res.public_directory, "public");
        assert_eq!(res.suggested_host, "my-cool-laravel.test");
        assert_eq!(res.dev_command, "php artisan serve");
    }

    #[test]
    fn test_detect_php_composer() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("api-service");
        fs::create_dir_all(&project_dir).expect("mkdir");
        fs::write(
            project_dir.join("composer.json"),
            r#"{"require": {"guzzlehttp/guzzle": "^7.0"}}"#,
        )
        .expect("write");

        let res = FrameworkDetector::detect(&project_dir);
        assert_eq!(res.framework, "php");
        assert_eq!(res.suggested_port, Some(8000));
        assert_eq!(res.public_directory, ".");
        assert_eq!(res.suggested_host, "api-service.test");
    }

    #[test]
    fn test_detect_nextjs() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("frontend-app");
        fs::create_dir_all(&project_dir).expect("mkdir");
        fs::write(
            project_dir.join("package.json"),
            r#"{"dependencies": {"next": "^14.0", "react": "^18.0"}}"#,
        )
        .expect("write");

        let res = FrameworkDetector::detect(&project_dir);
        assert_eq!(res.framework, "nextjs");
        assert_eq!(res.suggested_port, Some(3000));
        assert_eq!(res.suggested_host, "frontend-app.test");
        assert_eq!(res.dev_command, "npm run dev");
    }

    #[test]
    fn test_detect_vite() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("dashboard_ui");
        fs::create_dir_all(&project_dir).expect("mkdir");
        fs::write(
            project_dir.join("package.json"),
            r#"{"devDependencies": {"vite": "^5.0"}}"#,
        )
        .expect("write");

        let res = FrameworkDetector::detect(&project_dir);
        assert_eq!(res.framework, "vite");
        assert_eq!(res.suggested_port, Some(5173));
        assert_eq!(res.suggested_host, "dashboard-ui.test");
    }

    #[test]
    fn test_detect_node() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("backend-express");
        fs::create_dir_all(&project_dir).expect("mkdir");
        fs::write(
            project_dir.join("package.json"),
            r#"{"dependencies": {"express": "^4.18"}}"#,
        )
        .expect("write");

        let res = FrameworkDetector::detect(&project_dir);
        assert_eq!(res.framework, "node");
        assert_eq!(res.suggested_port, Some(3000));
    }

    #[test]
    fn test_detect_raw_php() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("simple-script");
        fs::create_dir_all(&project_dir).expect("mkdir");
        fs::write(project_dir.join("index.php"), "<?php phpinfo(); ?>").expect("write");

        let res = FrameworkDetector::detect(&project_dir);
        assert_eq!(res.framework, "php");
        assert_eq!(res.suggested_port, Some(8000));
    }

    #[test]
    fn test_detect_static() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("landing-page");
        fs::create_dir_all(&project_dir).expect("mkdir");
        fs::write(project_dir.join("index.html"), "<h1>Hello</h1>").expect("write");

        let res = FrameworkDetector::detect(&project_dir);
        assert_eq!(res.framework, "static");
        assert_eq!(res.suggested_port, None);
    }

    #[test]
    fn test_detect_nonexistent_or_empty() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("empty-folder");
        let res = FrameworkDetector::detect(&project_dir);
        assert_eq!(res.framework, "custom");
        assert_eq!(res.suggested_host, "empty-folder.test");
    }
}
