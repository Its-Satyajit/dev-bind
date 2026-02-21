use super::*;

#[test]
fn test_get_config_path_has_correct_suffix() {
    let path = get_config_path();
    assert!(path.ends_with("devbind/config.toml"));
}

#[test]
fn test_get_config_dir_is_parent_of_path() {
    let path = get_config_path();
    let dir = get_config_dir();
    assert_eq!(dir, path.parent().unwrap());
    assert!(dir.ends_with("devbind"));
}

#[test]
fn test_which_devbind_returns_a_string() {
    let bin = which_devbind();
    assert!(!bin.is_empty());
    // Generally returns "devbind" unless ~/.local/bin/devbind exists
    assert!(bin.contains("devbind"));
}
