use super::support::rejected;

#[test]
fn missing_comma_shows_earlier_source_without_claiming_exact_blame() {
    let report = rejected(
        "raven.appearance {
-- 2
-- 3
-- 4
-- 5
-- 6
border = {
width = 2
-- missing comma above
active = '#ffffff',
},
}
",
    );
    assert!(report.contains("Lua could not read"), "{report}");
    assert!(report.contains("missing comma"), "{report}");
    assert!(report.contains("8 | width = 2"), "{report}");
    assert!(report.contains(">   10 | active"), "{report}");
    assert!(report.contains("may be on an earlier line"), "{report}");
}

#[test]
fn invalid_border_names_the_field_value_and_allowed_range() {
    let report = rejected("raven.appearance { border = { width = -2 } }");
    for expected in [
        "A configuration setting is invalid",
        "appearance.border.width",
        "0 through 65535",
        "Received -2",
        "Technical details",
        "stack traceback",
    ] {
        assert!(report.contains(expected), "missing {expected}: {report}");
    }
}

#[test]
fn unknown_setting_lists_valid_choices() {
    let report = rejected("raven.appearance { border = { widht = 2 } }");
    assert!(report.contains("unknown setting"), "{report}");
    assert!(report.contains("widht"), "{report}");
    assert!(
        report.contains("Allowed settings here: width, active, inactive"),
        "{report}"
    );
}

#[test]
fn runtime_failure_is_not_mislabeled_as_invalid_settings() {
    let report = rejected("error('custom failure')");
    assert!(
        report.contains("The configuration stopped while running"),
        "{report}"
    );
    assert!(report.contains("custom failure"), "{report}");
    assert!(
        !report.contains("A configuration setting is invalid"),
        "{report}"
    );
}

#[test]
fn stray_name_is_highlighted_before_lua_detection_without_executing_repair() {
    let source = "\n".repeat(24)
        + "a\nraven.animations { resize_ms = 200 }\nerror('probe must not execute')\n";
    let report = rejected(&source);
    for expected in [
        "Possible stray name",
        "on line 25",
        ">   25 | a",
        "later, on line 26",
        "Your file was not changed",
    ] {
        assert!(report.contains(expected), "missing {expected}: {report}");
    }
}

#[test]
fn text_inside_comment_is_not_offered_as_a_repair() {
    let report = rejected("--[[\na\n]]\nraven.appearance {\n");
    assert!(!report.contains("Possible stray name"), "{report}");
    assert!(report.contains("Lua could not read"), "{report}");
}
