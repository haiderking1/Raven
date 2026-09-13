fn main() {
    let source = "src/runtime/config/dialog/window.c";
    println!("cargo:rerun-if-changed={source}");
    let gtk = pkg_config::Config::new()
        .atleast_version("4.0")
        .probe("gtk4")
        .expect("GTK4 development files are required for the configuration error window");
    cc::Build::new()
        .file(source)
        .includes(gtk.include_paths)
        .flag_if_supported("-std=c11")
        .warnings(true)
        .compile("raven_config_dialog");
}
