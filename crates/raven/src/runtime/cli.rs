use std::{error::Error, ffi::OsString};

pub(super) fn parse(
    args: impl Iterator<Item = OsString>,
) -> Result<Option<Vec<OsString>>, Box<dyn Error>> {
    let mut args = args.peekable();
    match args.next() {
        None => Ok(Some(Vec::new())),
        Some(arg) if arg == "--help" || arg == "-h" => {
            println!(
                "Raven direct-TTY Wayland compositor

Usage: raven [-- COMMAND [ARG...]]

Run as your normal login user on an active Linux VT.
Device access requires logind or seatd; do not run as root.

Super+Shift+Q    Exit Raven and return to the TTY
Ctrl+Alt+F1..F12 Switch virtual terminal

Waybar starts once per Raven session, including when COMMAND is provided.
Startup failures are logged and do not stop Raven or other startup entries.
Optional COMMAND launches in addition, with Raven's WAYLAND_DISPLAY.
Arguments are passed literally, without shell parsing.
Example: raven -- foot"
            );
            Ok(None)
        }
        Some(arg) if arg == "--" => {
            let command: Vec<_> = args.collect();
            if command.is_empty() {
                return Err("expected a command after --".into());
            }
            Ok(Some(command))
        }
        Some(arg) => Err(format!(
            "unknown argument {}; use --help or -- COMMAND",
            arg.to_string_lossy()
        )
        .into()),
    }
}
