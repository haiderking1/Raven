use std::process::ExitCode;

fn main() -> ExitCode {
    match raven::runtime::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("raven: {error}");
            ExitCode::FAILURE
        }
    }
}
