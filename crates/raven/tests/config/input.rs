use super::support::{ConfigFile, rejected};

#[test]
fn input_values_reject_invalid_ranges_types_and_profiles() {
    for (source, field, expected) in [
        (
            "raven.input { keyboard = { repeat_rate = -1 } }",
            "input.keyboard.repeat_rate",
            "0 to 1000",
        ),
        (
            "raven.input { keyboard = { repeat_delay = 60001 } }",
            "input.keyboard.repeat_delay",
            "0 to 60000",
        ),
        (
            "raven.input { keyboard = { repeat_rate = 2.5 } }",
            "input.keyboard.repeat_rate",
            "whole number",
        ),
        (
            "raven.input { mouse = { accel_profile = 'off' } }",
            "input.mouse.accel_profile",
            "flat",
        ),
    ] {
        let report = rejected(source);
        assert!(report.contains(field), "{report}");
        assert!(report.contains(expected), "{report}");
        assert!(
            report.contains("A configuration setting is invalid"),
            "{report}"
        );
    }
}

#[test]
fn input_boundaries_and_profiles_are_accepted_without_devices() {
    let file = ConfigFile::new();
    for profile in ["default", "flat", "adaptive"] {
        let source = format!(
            "raven.input {{ keyboard = {{ repeat_rate = 0, repeat_delay = 0 }}, mouse = {{ accel_profile = '{profile}' }} }}\nraven.input {{ keyboard = {{ repeat_rate = 1000, repeat_delay = 60000 }} }}"
        );
        let output = file.command(&source).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
