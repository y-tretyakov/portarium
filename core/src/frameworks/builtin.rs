use crate::models::Framework;

pub(crate) const KNOWN_PORTS: &[(u16, &str, bool)] = &[
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

pub(crate) fn builtin_frameworks() -> Vec<Framework> {
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
    fn builtin_contains_expected_entries() {
        let fws = builtin_frameworks();
        assert_eq!(fws.len(), KNOWN_PORTS.len());
        assert!(fws.iter().any(|f| f.name == "Vite"));
        assert!(fws.iter().any(|f| f.port == 5432 && !f.is_dev));
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
}
