//! Pure and tested PATH string editor for Windows.

pub struct PathEditor;

impl PathEditor {
    /// Split a Windows PATH string by semicolon, trimming spaces and ignoring empty items.
    pub fn split_path(path: &str) -> Vec<String> {
        path.split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Join list of paths into a single Windows PATH string.
    pub fn join_path(entries: &[String]) -> String {
        entries.join(";")
    }

    /// Check if path contains a specific directory (case-insensitive on Windows).
    pub fn contains(path: &str, dir: &str) -> bool {
        let normalized = dir.trim().trim_end_matches('\\').to_lowercase();
        Self::split_path(path).iter().any(|entry| {
            entry.trim_end_matches('\\').to_lowercase() == normalized
        })
    }

    /// Add directory to PATH. If prepend is true, adds to front. Avoids duplicates.
    pub fn add_path(current_path: &str, new_dir: &str, prepend: bool) -> String {
        let mut entries = Self::split_path(current_path);
        let normalized = new_dir.trim().trim_end_matches('\\').to_lowercase();

        // Remove any existing duplicate first
        entries.retain(|e| e.trim_end_matches('\\').to_lowercase() != normalized);

        if prepend {
            entries.insert(0, new_dir.trim().to_string());
        } else {
            entries.push(new_dir.trim().to_string());
        }

        Self::join_path(&entries)
    }

    /// Remove a directory from PATH.
    pub fn remove_path(current_path: &str, dir_to_remove: &str) -> String {
        let mut entries = Self::split_path(current_path);
        let normalized = dir_to_remove.trim().trim_end_matches('\\').to_lowercase();

        entries.retain(|e| e.trim_end_matches('\\').to_lowercase() != normalized);
        Self::join_path(&entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_remove_path() {
        let initial = "C:\\Windows;C:\\Windows\\System32";
        let added = PathEditor::add_path(initial, "C:\\Users\\dev\\.dotnet\\bin", true);
        assert!(added.starts_with("C:\\Users\\dev\\.dotnet\\bin;"));
        assert!(PathEditor::contains(&added, "C:\\Users\\dev\\.dotnet\\bin"));

        let removed = PathEditor::remove_path(&added, "C:\\Users\\dev\\.dotnet\\bin");
        assert_eq!(removed, initial);
    }
}
