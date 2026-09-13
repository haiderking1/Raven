pub(in crate::runtime) fn run_if_requested() -> Option<Result<(), Box<dyn std::error::Error>>> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--check-config")) {
        return None;
    }
    Some((|| {
        let path = match args.next() {
            Some(path) => path.into(),
            None => super::files::path()?,
        };
        if args.next().is_some() {
            return Err("usage: raven --check-config [path]".into());
        }
        let source = super::files::read(&path)?;
        super::lua::parse(&source, &path, Default::default())?;
        println!("Valid configuration: {}", path.display());
        Ok(())
    })())
}
