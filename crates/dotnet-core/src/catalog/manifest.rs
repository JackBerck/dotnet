use crate::error::Result;
use crate::models::ToolManifest;

pub struct CatalogLoader;

impl CatalogLoader {
    /// Loads all embedded tool manifests into memory.
    pub fn load_embedded() -> Result<Vec<ToolManifest>> {
        let raw_manifests = [
            include_str!("manifests/bun.json"),
            include_str!("manifests/composer.json"),
            include_str!("manifests/git.json"),
            include_str!("manifests/go.json"),
            include_str!("manifests/mysql.json"),
            include_str!("manifests/nginx.json"),
            include_str!("manifests/node.json"),
            include_str!("manifests/php.json"),
            include_str!("manifests/postgresql.json"),
        ];

        let mut list = Vec::with_capacity(raw_manifests.len());
        for raw in raw_manifests {
            let manifest: ToolManifest = serde_json::from_str(raw)?;
            list.push(manifest);
        }

        Ok(list)
    }

    /// Find an embedded tool manifest by ID.
    pub fn find_embedded(id: &str) -> Result<Option<ToolManifest>> {
        let manifests = Self::load_embedded()?;
        Ok(manifests.into_iter().find(|m| m.id.eq_ignore_ascii_case(id)))
    }
}
