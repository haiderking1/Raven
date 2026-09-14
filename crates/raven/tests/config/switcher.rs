use super::support::{ConfigFile, rejected};
#[test]
fn application_switcher_actions_require_an_alt_held_binding() {
    let directory = ConfigFile::new();
    let result = directory
        .command(
            r#"raven.bind("Alt+Tab", "next_app")
raven.bind("Alt+Shift+Tab", "previous_app")"#,
        )
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(rejected("raven.bind('Super+Tab', 'next_app')").contains("must include Alt"));
}
