use crate::models::Framework;

const KNOWN_PORTS: &[(u16, &str, bool)] = &[
    (3000, "React", true),
    (3001, "React", true),
    (4000, "Node", true),
    (4200, "Angular", true),
    (5173, "Vite", true),
    (5174, "Vite", true),
    (8000, "Django", true),
    (8080, "HTTP", true),
    (8888, "Jupyter", true),
    (5432, "Postgres", false),
    (3306, "MySQL", false),
    (6379, "Redis", false),
    (27017, "Mongo", false),
    (9000, "PHP", true),
    (1420, "Tauri", true),
    (4173, "Vite", true),
    (2000, "Node", true),
    (8443, "HTTPS", true),
];

pub fn get_framework(port: u16) -> Option<String> {
    KNOWN_PORTS
        .iter()
        .find(|(p, _, _)| *p == port)
        .map(|(_, name, _)| name.to_string())
}

pub fn is_dev_port(port: u16) -> bool {
    KNOWN_PORTS
        .iter()
        .any(|(p, _, is_dev)| *p == port && *is_dev)
}

pub fn get_all_frameworks() -> Vec<Framework> {
    KNOWN_PORTS
        .iter()
        .map(|(port, name, is_dev)| Framework {
            port: *port,
            name: name.to_string(),
            is_dev: *is_dev,
        })
        .collect()
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
        assert_eq!(frameworks.len(), KNOWN_PORTS.len());
        assert!(frameworks.iter().any(|f| f.name == "Vite"));
        assert!(frameworks.iter().any(|f| f.port == 5432 && !f.is_dev));
    }

    #[test]
    fn framework_serialization() {
        let fw = Framework {
            port: 3000,
            name: "React".into(),
            is_dev: true,
        };
        let json = serde_json::to_string(&fw).unwrap();
        assert!(json.contains("3000"));
        assert!(json.contains("React"));
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
