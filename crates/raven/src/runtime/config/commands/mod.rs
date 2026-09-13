mod executable;
use std::ffi::OsString;

pub(super) fn validate(name: &str, argv: &[OsString]) -> Result<(), String> {
    let program = argv
        .first()
        .ok_or_else(|| format!("{name}: provide a command, for example {{ 'foot' }}."))?;
    executable::check(program).map_err(|reason| format!("{name}: cannot launch {program:?}. {reason}
Check the spelling, install the program, or use its executable path. Raven does not substitute another program."))
}
