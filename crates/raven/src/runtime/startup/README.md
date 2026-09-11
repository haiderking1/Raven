# Session startup

`StartupEntry` holds a literal `Vec<OsString>` argv, an optional `PathBuf` cwd, and a `BTreeMap<OsString, OsString>` of environment overrides. argv includes the executable at index zero. Arguments may contain spaces or non-UTF-8 bytes. Raven does not split, expand, or interpret them as shell syntax. cwd is passed to the child unchanged; a relative path is relative to Raven's working directory. Unspecified environment variables are inherited.

`StartupPlan { entries }` is reusable runtime settings, independent of Lua or another parser. Its default contains one entry, `waybar`. An empty entries list disables startup applications. Constructing a plan has no side effects.

Consume a plan with `StartupPlan::validate()` before backend acquisition. This produces an immutable `ValidatedStartupPlan`, retaining errors for individual entries. Validation rejects missing or empty executables, NUL bytes, empty cwd paths, and invalid environment names. It does not probe executable availability or directory access. Those can change before spawn and are checked by the OS when launching.

After the Wayland socket, satellite reservation, and backend are ready, call `Clients::start_startup(&plan)`. It logs each invalid or unlaunchable entry and continues in order. The first call consumes the owner's startup opportunity, even for an empty plan or failed entries. Further calls, including calls with replacement settings after a future reload, do nothing. A new Clients owner has a new startup lifetime. There is no implicit reload, retry, respawn, or Lua parsing.

`Clients::new` does not load or execute startup settings. The runtime explicitly supplies the validated plan from runtime::settings::Settings. An explicit CLI command, such as `raven -- foot`, launches in addition to Waybar and retains its existing fatal launch-error behavior. Runtime keybinding launches through `Clients::spawn` are not subject to the startup guard.

Entry environment overrides apply only to their child. Raven then sets WAYLAND_DISPLAY, XDG_SESSION_TYPE, and XDG_CURRENT_DESKTOP, and removes WAYLAND_SOCKET. XDG_RUNTIME_DIR stays the same as Raven's so relative socket names resolve correctly. It supplies the managed DISPLAY and removes XAUTHORITY when the satellite is available; otherwise it removes DISPLAY. No process-global environment, systemd service, or user configuration changes occur.

Successful startup children enter the existing Clients child collection. SIGCHLD triggers the existing nonblocking reap path, without timers or idle polling. Raven does not automatically restart exited children. Shutdown drops the backend first, then kills and waits only for still-running direct children owned by Clients. Connected clients and unrelated processes are not cleanup targets; daemonized descendants are not adopted or managed.
