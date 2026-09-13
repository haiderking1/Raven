use super::support::ConfigFile;
use std::{fs, os::unix::fs::PermissionsExt};

#[test]
fn missing_recovery_commands_reject_the_candidate() {
    let fixture = ConfigFile::new();
    for name in ["terminal", "launcher"] {
        let source = format!("raven.commands {{ {name} = {{ 'foote' }} }}");
        let output = fixture.command(&source).env("PATH", "").output().unwrap();
        assert!(!output.status.success());
        let error = String::from_utf8(output.stderr).unwrap();
        for text in [
            format!("commands.{name}"),
            "foote".into(),
            "No executable with that name".into(),
            "Check the spelling".into(),
        ] {
            assert!(error.contains(&text), "{error}");
        }
    }
}

#[test]
fn search_checks_permissions_and_relative_paths_without_executing_commands() {
    let fixture = ConfigFile::new();
    for directory in ["bad", "good"] {
        fs::create_dir(fixture.0.join(directory)).unwrap();
        let path = fixture.0.join(directory).join("terminal-tool");
        fs::write(&path, "#!/bin/sh\necho ran > executed\n").unwrap();
        fs::set_permissions(
            path,
            fs::Permissions::from_mode(if directory == "good" { 0o700 } else { 0o600 }),
        )
        .unwrap();
    }
    for (program, search, accepted) in [
        ("terminal-tool", "bad", false),
        ("terminal-tool", "bad:good", true),
        ("./good/terminal-tool", "", true),
    ] {
        // Stop evaluation after validation: this check needs no installed cursor
        // theme, and never launches a compositor or executes the test program.
        let source = format!(
            "raven.commands {{ terminal = {{ '{program}' }} }}\nerror('command-check-passed')"
        );
        let output = fixture
            .command(&source)
            .env("PATH", search)
            .output()
            .unwrap();
        let error = String::from_utf8(output.stderr).unwrap();
        assert_eq!(
            error.contains("The configuration stopped while running"),
            accepted,
            "{error}"
        );
        if !accepted {
            assert!(error.contains("not executable"), "{error}");
        }
        assert!(!fixture.0.join("executed").exists());
    }
}
