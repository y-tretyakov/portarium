mod builtin;
mod registry;

use once_cell::sync::Lazy;
use registry::FrameworkRegistry;

use crate::models::Framework;

static REGISTRY: Lazy<FrameworkRegistry> = Lazy::new(FrameworkRegistry::new);

pub fn get_framework(port: u16) -> Option<String> {
    REGISTRY.get(port).map(|fw| fw.name.clone())
}

pub fn is_dev_port(port: u16) -> bool {
    REGISTRY.is_dev(port)
}

pub fn get_all_frameworks() -> Vec<Framework> {
    REGISTRY.all().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_known_framework() {
        assert_eq!(get_framework(5173), Some("Vite".into()));
        assert_eq!(get_framework(3000), Some("React".into()));
        assert_eq!(get_framework(5432), Some("Postgres".into()));
    }

    #[test]
    fn returns_none_for_unknown_port() {
        assert_eq!(get_framework(9999), None);
        assert_eq!(get_framework(0), None);
    }

    #[test]
    fn identifies_dev_ports() {
        assert!(is_dev_port(3000));
        assert!(is_dev_port(5173));
        assert!(!is_dev_port(5432));
        assert!(!is_dev_port(9999));
    }

    #[test]
    fn get_all_returns_complete_list() {
        let frameworks = get_all_frameworks();
        assert!(!frameworks.is_empty());
        assert!(frameworks.iter().any(|f| f.name == "Vite"));
        assert!(frameworks.iter().any(|f| f.port == 5432 && !f.is_dev));
    }

    proptest::proptest! {
        #[test]
        fn known_frameworks_all_have_valid_ports(port in 1u16..65535u16) {
            if let Some(name) = get_framework(port) {
                assert!(!name.is_empty());
            }
        }
    }
}
