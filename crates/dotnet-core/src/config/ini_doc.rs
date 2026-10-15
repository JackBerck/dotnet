//! Robust INI document editor preserving existing comments, whitespace, and formatting.

pub struct IniDocument {
    lines: Vec<String>,
}

impl IniDocument {
    pub fn parse(content: &str) -> Self {
        Self {
            lines: content.lines().map(|s| s.to_string()).collect(),
        }
    }
}

impl std::fmt::Display for IniDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.lines {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl IniDocument {
    /// Retrieve setting value (excluding commented lines).
    pub fn get_value(&self, key: &str) -> Option<String> {
        for line in &self.lines {
            let trimmed = line.trim();
            if trimmed.starts_with(';') || trimmed.starts_with('#') {
                continue;
            }
            if let Some(pos) = trimmed.find('=') {
                let (k, v) = trimmed.split_at(pos);
                if k.trim() == key {
                    return Some(v[1..].trim().to_string());
                }
            }
        }
        None
    }

    /// Set or update key value. Uncomments if found commented, or updates active line, or appends.
    pub fn set_value(&mut self, key: &str, value: &str) {
        // Try finding active line first
        for line in self.lines.iter_mut() {
            let trimmed = line.trim();
            if !trimmed.starts_with(';') && !trimmed.starts_with('#') {
                if let Some(pos) = trimmed.find('=') {
                    let k = trimmed[..pos].trim();
                    if k == key {
                        *line = format!("{} = {}", key, value);
                        return;
                    }
                }
            }
        }

        // Try finding commented line to uncomment
        for line in self.lines.iter_mut() {
            let trimmed = line.trim();
            if trimmed.starts_with(';') || trimmed.starts_with('#') {
                let uncommented = trimmed[1..].trim();
                if let Some(pos) = uncommented.find('=') {
                    let k = uncommented[..pos].trim();
                    if k == key {
                        *line = format!("{} = {}", key, value);
                        return;
                    }
                }
            }
        }

        // Otherwise append at the end
        self.lines.push(format!("{} = {}", key, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ini_update() {
        let sample = ";extension=curl\r\nmemory_limit = 128M\r\n";
        let mut doc = IniDocument::parse(sample);
        assert_eq!(doc.get_value("memory_limit"), Some("128M".to_string()));

        doc.set_value("memory_limit", "512M");
        assert_eq!(doc.get_value("memory_limit"), Some("512M".to_string()));

        doc.set_value("extension", "curl");
        assert_eq!(doc.get_value("extension"), Some("curl".to_string()));
    }
}
