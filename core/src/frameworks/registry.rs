use std::path::Path;

use serde::Deserialize;

use crate::error::Result;
use crate::frameworks::builtin::builtin_frameworks;
use crate::models::Framework;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TomlConfig {
    frameworks: TomlFrameworks,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TomlFrameworks {
    custom: Vec<TomlFramework>,
}

#[derive(Debug, Deserialize)]
struct TomlFramework {
    port: u16,
    name: String,
    dev: bool,
}

impl From<TomlFramework> for Framework {
    fn from(tf: TomlFramework) -> Self {
        Framework {
            port: tf.port,
            name: tf.name,
            is_dev: tf.dev,
        }
    }
}

pub struct FrameworkRegistry {
    entries: Vec<Framework>,
}

impl FrameworkRegistry {
    pub fn new() -> Self {
        FrameworkRegistry {
            entries: builtin_frameworks(),
        }
    }

    #[allow(dead_code)]
    pub fn load_toml(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let toml: TomlConfig =
            toml::from_str(&content).map_err(|e| crate::error::Error::Parse(e.to_string()))?;

        let mut entries = builtin_frameworks();
        for tf in toml.frameworks.custom {
            entries.push(Framework::from(tf));
        }

        Ok(FrameworkRegistry { entries })
    }

    pub fn get(&self, port: u16) -> Option<&Framework> {
        self.entries.iter().find(|fw| fw.port == port)
    }

    pub fn is_dev(&self, port: u16) -> bool {
        self.entries.iter().any(|fw| fw.port == port && fw.is_dev)
    }

    pub fn all(&self) -> &[Framework] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn registry_contains_builtin() {
        let reg = FrameworkRegistry::new();
        assert!(reg.get(5173).is_some());
        assert!(reg.get(5432).is_some());
        assert!(reg.get(9999).is_none());
    }

    #[test]
    fn registry_is_dev_delegates() {
        let reg = FrameworkRegistry::new();
        assert!(reg.is_dev(3000));
        assert!(!reg.is_dev(5432));
        assert!(!reg.is_dev(9999));
    }

    #[test]
    fn registry_all_returns_builtin_count() {
        let reg = FrameworkRegistry::new();
        let builtin_count = builtin_frameworks().len();
        assert_eq!(reg.all().len(), builtin_count);
    }

    #[test]
    fn load_toml_adds_custom_frameworks() {
        let toml_content = r#"
[frameworks]
custom = [
  { port = 1234, name = "MyService", dev = true },
  { port = 5678, name = "MyDB", dev = false },
]
"#;
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "{}", toml_content).unwrap();
        let reg = FrameworkRegistry::load_toml(tmp.path()).unwrap();

        assert_eq!(reg.get(1234).unwrap().name, "MyService");
        assert!(reg.is_dev(1234));
        assert_eq!(reg.get(5678).unwrap().name, "MyDB");
        assert!(!reg.is_dev(5678));
        assert!(reg.get(5173).is_some()); // builtin still present
    }

    #[test]
    fn load_toml_invalid_path_returns_error() {
        let result = FrameworkRegistry::load_toml(Path::new("/nonexistent/path.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn load_toml_malformed_returns_error() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "not valid toml {{").unwrap();
        let result = FrameworkRegistry::load_toml(tmp.path());
        assert!(result.is_err());
    }
}
