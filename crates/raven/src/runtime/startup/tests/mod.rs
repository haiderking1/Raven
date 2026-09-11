mod fixture;

use super::super::{Clients, spawn};
use crate::runtime::startup::{StartupEntry, StartupPlan};
use fixture::Fixture;
use std::{fs, os::unix::fs::PermissionsExt};

fn clients() -> Clients {
    // No satellite probe, socket reservation, backend, or GUI in this regression.
    Clients {
        socket: "raven-startup-test-socket".into(),
        children: Default::default(),
        startup_started: false,
        satellite: None,
    }
}

fn finish(clients: &mut Clients, expected: usize) {
    assert_eq!(clients.children.0.len(), expected);
    for child in &mut clients.children.0 {
        assert!(child.wait().unwrap().success());
    }
    // The same reap path used by SIGCHLD releases the cached exit statuses.
    clients.reap();
    assert!(clients.children.0.is_empty());
}

#[test]
fn startup_is_once_per_owner_with_literal_argv_environment_cwd_and_isolated_failures() {
    assert_eq!(StartupPlan::default().entries[0].argv, ["waybar"]);
    let fixture = Fixture::new();
    let inherited = std::env::var_os("RAVEN_STARTUP_TEST");
    let missing = fixture.directory.join("initially-missing");
    let mut bad_cwd = fixture.entry("cwd-retry");
    bad_cwd.cwd = Some(fixture.directory.join("initially-missing-directory"));
    let plan = StartupPlan {
        entries: vec![
            fixture.entry("before"),
            StartupEntry {
                argv: vec![missing.clone().into_os_string()],
                ..Default::default()
            },
            StartupEntry::default(),
            bad_cwd,
            fixture.entry("after"),
        ],
    }
    .validate();
    assert!(plan.entries()[2].is_err());
    let mut owner = clients();
    assert!(owner.children.0.is_empty());
    owner.start_startup(&plan);
    finish(&mut owner, 2);
    fixture.assert_output("before", "unset", "wrong-authority");
    fixture.assert_output("after", "unset", "wrong-authority");

    // Repair both runtime failures. Repeated calls still must not retry either.
    fs::write(&missing, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&missing, fs::Permissions::from_mode(0o700)).unwrap();
    fs::create_dir(fixture.directory.join("initially-missing-directory")).unwrap();
    owner.start_startup(&plan);
    let replacement = StartupPlan {
        entries: vec![fixture.entry("replacement")],
    }
    .validate();
    owner.start_startup(&replacement);
    assert!(owner.children.0.is_empty());
    fixture.assert_output("before", "unset", "wrong-authority");
    fixture.assert_output("after", "unset", "wrong-authority");
    assert!(!fixture.directory.join("replacement.out").exists());

    // A new owner has a new session lifetime; a plan has no global launch state.
    let mut next_owner = clients();
    next_owner.start_startup(&replacement);
    finish(&mut next_owner, 1);
    fixture.assert_output("replacement", "unset", "wrong-authority");

    // Exercise the shared spawn path with a managed X11 display, without X11.
    let x11 = fixture.entry("managed-x11");
    let child = spawn::client(
        &x11.argv,
        x11.cwd.as_deref(),
        &x11.env,
        &owner.socket,
        Some(":raven-test".as_ref()),
    )
    .unwrap()
    .unwrap();
    owner.children.track(child);
    finish(&mut owner, 1);
    fixture.assert_output("managed-x11", ":raven-test", "unset");
    assert_eq!(std::env::var_os("RAVEN_STARTUP_TEST"), inherited);
}
