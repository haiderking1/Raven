//! Anchor cleanup to open directories and the inode we created, not a later path.
use super::directory::Directory;
use std::{
    ffi::CString,
    fs::File,
    io,
    os::{fd::AsRawFd, unix::fs::MetadataExt},
    sync::Arc,
};

pub(super) struct Entry {
    directory: Arc<Directory>,
    name: CString,
    // Keep the inode pinned, even if somebody replaces the directory entry.
    inode: Option<File>,
    device: u64,
    number: u64,
}

impl Entry {
    pub(super) fn create(directory: Arc<Directory>, name: &str) -> io::Result<Self> {
        let name = CString::new(name)?;
        let inode = directory.open(&name, libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL, 0o644)?;
        let metadata = inode.metadata()?;
        Ok(Self {
            directory,
            name,
            device: metadata.dev(),
            number: metadata.ino(),
            inode: Some(inode),
        })
    }

    pub(super) fn socket(directory: Arc<Directory>, name: &str) -> io::Result<Self> {
        let metadata = std::fs::symlink_metadata(directory.bind_path(name))?;
        let name = CString::new(name)?;
        if metadata.mode() & libc::S_IFMT != libc::S_IFSOCK {
            return Err(io::Error::other(
                "X11 socket path was replaced during reservation",
            ));
        }
        // Establish rollback before opening the inode pin. EMFILE after bind must
        // not leave a stale socket behind. The snapshot is used only for that rollback.
        let mut entry = Self {
            directory,
            name,
            device: metadata.dev(),
            number: metadata.ino(),
            inode: None,
        };
        let inode = entry.directory.open(&entry.name, libc::O_PATH, 0)?;
        let pinned = inode.metadata()?;
        if pinned.dev() != entry.device || pinned.ino() != entry.number {
            return Err(io::Error::other(
                "X11 socket was replaced while pinning its inode",
            ));
        }
        entry.inode = Some(inode);
        Ok(entry)
    }

    pub(super) fn file(&mut self) -> &mut File {
        self.inode
            .as_mut()
            .expect("lock entry owns a writable file")
    }

    pub(super) fn visible(&self) -> bool {
        self.directory.visible() && self.unchanged()
    }

    fn unchanged(&self) -> bool {
        let mut current = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: fstatat initializes current on success; no symlinks are followed.
        let result = unsafe {
            libc::fstatat(
                self.directory.file.as_raw_fd(),
                self.name.as_ptr(),
                current.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result != 0 {
            return false;
        }
        let current = unsafe { current.assume_init() };
        self.device == current.st_dev && self.number == current.st_ino
    }
}

impl Drop for Entry {
    fn drop(&mut self) {
        if self.unchanged() {
            // SAFETY: both the directory and terminated entry name remain owned.
            unsafe {
                libc::unlinkat(self.directory.file.as_raw_fd(), self.name.as_ptr(), 0);
            }
        }
    }
}
