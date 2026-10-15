//! Post-installation operations runner (INI configuration, copy-if-missing, shims).

use crate::config::IniDocument;
use crate::error::Result;
use crate::models::ToolManifest;
use std::fs;
use std::path::Path;

pub struct PostInstallRunner;

impl PostInstallRunner {
    /// Execute post-installation operations defined in the tool manifest.
    pub fn run<P: AsRef<Path>>(manifest: &ToolManifest, install_dir: P) -> Result<()> {
        let install_dir = install_dir.as_ref();

        for op in &manifest.post_install {
            let op_name = op.op.to_lowercase();
            match op_name.as_str() {
                "copyifmissing" => {
                    if let (Some(from), Some(to)) = (&op.from, &op.to) {
                        let source = install_dir.join(from);
                        let dest = install_dir.join(to);
                        if source.exists() && !dest.exists() {
                            fs::copy(&source, &dest)?;
                            tracing::info!("Post-install: Copied {} to {}", from, to);
                        }
                    }
                }
                "iniset" => {
                    if let (Some(file), Some(key), Some(value)) = (&op.file, &op.key, &op.value) {
                        let ini_path = install_dir.join(file);
                        if ini_path.exists() {
                            let content = fs::read_to_string(&ini_path)?;
                            let mut doc = IniDocument::parse(&content);
                            doc.set_value(key, value);
                            fs::write(&ini_path, doc.to_string())?;
                            tracing::info!("Post-install: Set INI {} = {}", key, value);
                        }
                    }
                }
                "inienableextensions" => {
                    if let (Some(file), Some(names)) = (&op.file, &op.names) {
                        let ini_path = install_dir.join(file);
                        if ini_path.exists() {
                            let content = fs::read_to_string(&ini_path)?;
                            let mut doc = IniDocument::parse(&content);
                            for ext in names {
                                doc.enable_extension(ext);
                            }
                            fs::write(&ini_path, doc.to_string())?;
                            tracing::info!(
                                "Post-install: Enabled {} extensions in {}",
                                names.len(),
                                file
                            );
                        }
                    }
                }
                "writeshim" => {
                    if let (Some(file), Some(value)) = (&op.file, &op.value) {
                        let shim_path = install_dir.join(file);
                        fs::write(&shim_path, value)?;
                        tracing::info!("Post-install: Written shim at {}", shim_path.display());
                    }
                }
                _ => {
                    tracing::warn!("Unknown post-install operation: {}", op.op);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PostInstallOp, ToolLayout};

    #[test]
    fn test_post_install_copy_and_ini() {
        let temp_dir = tempfile::tempdir().unwrap();
        let install_path = temp_dir.path();

        // Create sample source file
        let ini_dev = install_path.join("php.ini-development");
        fs::write(&ini_dev, ";extension=curl\r\nmemory_limit = 128M\r\n").unwrap();

        let manifest = ToolManifest {
            schema_version: 1,
            id: "php".into(),
            display_name: "PHP".into(),
            category: "language".into(),
            homepage: None,
            license: None,
            platforms: vec![],
            prerequisites: vec![],
            allowed_hosts: vec![],
            versions: vec![],
            layout: ToolLayout::default(),
            post_install: vec![
                PostInstallOp {
                    op: "copyIfMissing".into(),
                    from: Some("php.ini-development".into()),
                    to: Some("php.ini".into()),
                    file: None,
                    key: None,
                    value: None,
                    names: None,
                },
                PostInstallOp {
                    op: "iniSet".into(),
                    from: None,
                    to: None,
                    file: Some("php.ini".into()),
                    key: Some("memory_limit".into()),
                    value: Some("512M".into()),
                    names: None,
                },
                PostInstallOp {
                    op: "iniEnableExtensions".into(),
                    from: None,
                    to: None,
                    file: Some("php.ini".into()),
                    key: None,
                    value: None,
                    names: Some(vec!["curl".into(), "mbstring".into()]),
                },
            ],
            verify: None,
        };

        PostInstallRunner::run(&manifest, install_path).unwrap();

        let php_ini = install_path.join("php.ini");
        assert!(php_ini.exists());

        let final_content = fs::read_to_string(&php_ini).unwrap();
        let doc = IniDocument::parse(&final_content);
        assert_eq!(doc.get_value("memory_limit"), Some("512M".into()));
        assert!(doc.is_extension_enabled("curl"));
        assert!(doc.is_extension_enabled("mbstring"));
    }
}
