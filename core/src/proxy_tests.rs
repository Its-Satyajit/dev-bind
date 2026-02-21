use super::*;
use crate::config::RouteConfig;

#[test]
fn test_build_route_map_lowercases_domains() {
    let mut config = DevBindConfig::default();
    config.routes.push(RouteConfig {
        domain: "MyApp.Test".to_string(),
        port: 3000,
        ephemeral: false,
    });
    config.routes.push(RouteConfig {
        domain: "Another-APP.TEST".to_string(),
        port: 8080,
        ephemeral: true,
    });

    let map = build_route_map(&config);

    assert_eq!(map.len(), 2);
    assert_eq!(map.get("myapp.test"), Some(&3000));
    assert_eq!(map.get("another-app.test"), Some(&8080));
    // Original cased domains shouldn't exist
    assert!(!map.contains_key("MyApp.Test"));
}

#[test]
fn test_build_route_map_empty() {
    let config = DevBindConfig::default();
    let map = build_route_map(&config);
    assert!(map.is_empty());
}
