//! FFI for statvfs functions
//!
//! See the `vfs::Statvfs` struct for some rusty wrappers

pub use self::vfs::{statvfs, fstatvfs};

pub mod vfs {
    //! Structs related to the `statvfs` and `fstatvfs` functions
    //!
    //! The `Statvfs` struct has some wrappers methods around the `statvfs` and
    //! `fstatvfs` calls.

    use {Errno, Result, NixPath};

    use libc;
    use std::os::unix::io::AsRawFd;
    use std::mem;

    pub struct Statvfs(libc::statvfs);

    impl Default for Statvfs {
        /// Create a statvfs object initialized to all zeros
        fn default() -> Self {
            unsafe { Statvfs(mem::zeroed()) }
        }
    }

    impl Statvfs {
        /// The posix statvfs struct
        ///
        /// http://linux.die.net/man/2/statvfs
        fn fields(&self) -> &libc::statvfs {
            &self.0
        }

        /// Create a new `Statvfs` object and fill it with information about
        /// the mount that contains `path`
        pub fn for_path<P: ?Sized + NixPath>(path: &P) -> Result<Statvfs> {
            let mut stat = Statvfs::default();
            let res = statvfs(path, &mut stat);
            res.map(|_| stat)
        }

        /// Replace information in this struct with information about `path`
        pub fn update_with_path<P: ?Sized + NixPath>(&mut self, path: &P) -> Result<()> {
            statvfs(path, self)
        }

        /// Create a new `Statvfs` object and fill it with information from fd
        pub fn for_fd<T: AsRawFd>(fd: &T) -> Result<Statvfs> {
            let mut stat = Statvfs::default();
            let res = fstatvfs(fd, &mut stat);
            res.map(|_| stat)
        }

        /// Replace information in this struct with information about `fd`
        pub fn update_with_fd<T: AsRawFd>(&mut self, fd: &T) -> Result<()> {
            fstatvfs(fd, self)
        }
    }

    /// Fill an existing `Statvfs` object with information about the `path`
    pub fn statvfs<P: ?Sized + NixPath>(path: &P, stat: &mut Statvfs) -> Result<()> {
        let res = try!(path.with_nix_path(|path| unsafe {
            libc::statvfs(path.as_ptr(), &mut stat.0 as *mut libc::statvfs)
        }));

        Errno::result(res).map(drop)
    }

    /// Fill an existing `Statvfs` object with information about `fd`
    pub fn fstatvfs<T: AsRawFd>(fd: &T, stat: &mut Statvfs) -> Result<()> {
        unsafe {
            Errno::result(libc::fstatvfs(fd.as_raw_fd(), &mut stat.0 as *mut libc::statvfs)).map(drop)
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use sys::statvfs::*;

    #[test]
    fn statvfs_call() {
        let mut stat = vfs::Statvfs::default();
        statvfs("/".as_bytes(), &mut stat).unwrap()
    }

    #[test]
    fn fstatvfs_call() {
        let mut stat = vfs::Statvfs::default();
        let root = File::open("/").unwrap();
        fstatvfs(&root, &mut stat).unwrap()
    }
}
