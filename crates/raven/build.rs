fn main() {
    let source = "src/runtime/config/dialog/window.c";
    println!("cargo:rerun-if-changed={source}");
    let gtk = pkg_config::Config::new()
        .atleast_version("4.0")
        .probe("gtk4")
        .expect("GTK4 development files are required for the configuration error window");
    let native = "src/desktop/switcher/artwork/native";
    println!("cargo:rerun-if-changed={native}");
    let gio = pkg_config::Config::new()
        .probe("gio-unix-2.0")
        .expect("GIO desktop-entry support is required");
    println!("cargo:rerun-if-changed=src/desktop/screenshot/native");
    cc::Build::new()
        .files([
            "src/desktop/screenshot/native/image.c",
            "src/desktop/screenshot/native/hud.c",
        ])
        .files(
            [
                "catalogue.c",
                "companions.c",
                "icons.c",
                "paint.c",
                "text.c",
            ]
            .map(|name| format!("{native}/{name}")),
        )
        .includes(gio.include_paths)
        .file(source)
        .includes(gtk.include_paths)
        .flag_if_supported("-std=c11")
        .warnings(true)
        .compile("raven_config_dialog");
}
