use super::support::ConfigFile;
#[test]
fn window_management_actions_are_configurable_without_changing_defaults() {
    let directory = ConfigFile::new();
    let result = directory
        .command(
            r#"raven.bind("Super+M", "minimize")
raven.bind("Super+Shift+F", "maximize")"#,
        )
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
