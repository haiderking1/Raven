mod ownership;

use std::{
    error::Error,
    fs,
    os::{
        linux::net::SocketAddrExt,
        unix::net::{SocketAddr, UnixListener},
    },
    path::{Path, PathBuf},
};

pub(crate) struct Sockets {
    paths: Vec<PathBuf>,
    abstract_name: String,
}

impl Sockets {
    pub fn owned(display: &str, wayland: &Path) -> Result<Self, Box<dyn Error>> {
        let number: u16 = display
            .strip_prefix(':')
            .ok_or("invalid reserved DISPLAY")?
            .parse()?;
        let paths = vec![
            PathBuf::from(format!("/tmp/.X11-unix/X{number}")),
            PathBuf::from(format!("/tmp/.X{number}-lock")),
            wayland.to_owned(),
            wayland.with_extension("lock"),
        ];
        for path in &paths {
            fs::symlink_metadata(path)?;
        }
        ownership::record(
            number,
            wayland.parent().ok_or("Wayland socket has no runtime")?,
        )?;
        Ok(Self {
            paths,
            abstract_name: format!("/tmp/.X11-unix/X{number}"),
        })
    }

    pub fn assert_removed(&self) -> Result<(), Box<dyn Error>> {
        for path in &self.paths {
            match fs::symlink_metadata(path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                result => {
                    return Err(format!(
                        "owned socket/lock remained at {}: {result:?}",
                        path.display()
                    )
                    .into());
                }
            }
        }
        // Rebinding also proves that no orphan Xwayland retains the inherited
        // abstract listener after the filesystem names have been removed.
        let address = SocketAddr::from_abstract_name(self.abstract_name.as_bytes())?;
        let _released = UnixListener::bind_addr(&address)?;
        Ok(())
    }
}
