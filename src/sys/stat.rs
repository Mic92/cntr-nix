pub use libc::dev_t;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use libc::{stat64 as FileStat};
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub use libc::{stat as FileStat};

use {Result, NixPath};
use errno::Errno;
use fcntl::AtFlags;
use libc::{self, mode_t};
use std::mem;
use std::os::unix::io::RawFd;

#[cfg(any(target_os = "linux", target_os = "android"))]
use libc::{fstat64, fstatat64, stat64, lstat64};
#[cfg(not(any(target_os = "linux", target_os = "android")))]
use libc::{fstat as fstat64, fstatat as fstatat64, stat as stat64, lstat as lstat64};

pub use self::linux::*;

libc_bitflags!(
    pub struct SFlag: mode_t {
        S_IFIFO;
        S_IFCHR;
        S_IFDIR;
        S_IFBLK;
        S_IFREG;
        S_IFLNK;
        S_IFSOCK;
        S_IFMT;
    }
);

libc_bitflags! {
    pub struct Mode: mode_t {
        S_IRWXU;
        S_IRUSR;
        S_IWUSR;
        S_IXUSR;
        S_IRWXG;
        S_IRGRP;
        S_IWGRP;
        S_IXGRP;
        S_IRWXO;
        S_IROTH;
        S_IWOTH;
        S_IXOTH;
        S_ISUID as mode_t;
        S_ISGID as mode_t;
        S_ISVTX as mode_t;
    }
}

pub fn mknod<P: ?Sized + NixPath>(path: &P, kind: SFlag, perm: Mode, dev: dev_t) -> Result<()> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            libc::mknod(cstr.as_ptr(), kind.bits | perm.bits() as mode_t, dev)
        }
    }));

    Errno::result(res).map(drop)
}

/// Create a special or ordinary file
/// ([posix specification](http://pubs.opengroup.org/onlinepubs/9699919799/functions/mknod.html)).
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub fn mknodat<P: ?Sized + NixPath>(dirfd: &RawFd, path: &P, kind: SFlag, perm: Mode, dev: dev_t) -> Result<()> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            libc::mknodat(*dirfd, cstr.as_ptr(), kind.bits | perm.bits() as mode_t, dev)
        }
    }));

    Errno::result(res).map(drop)
}

#[cfg(target_os = "linux")]
pub fn major(dev: dev_t) -> u64 {
    ((dev >> 32) & 0xffff_f000) |
    ((dev >>  8) & 0x0000_0fff)
}

#[cfg(target_os = "linux")]
pub fn minor(dev: dev_t) -> u64 {
    ((dev >> 12) & 0xffff_ff00) |
    ((dev      ) & 0x0000_00ff)
}

#[cfg(target_os = "linux")]
pub fn makedev(major: u64, minor: u64) -> dev_t {
    ((major & 0xffff_f000) << 32) |
    ((major & 0x0000_0fff) <<  8) |
    ((minor & 0xffff_ff00) << 12) |
     (minor & 0x0000_00ff)
}

pub fn umask(mode: Mode) -> Mode {
    let prev = unsafe { libc::umask(mode.bits() as mode_t) };
    Mode::from_bits(prev).expect("[BUG] umask returned invalid Mode")
}

pub fn stat<P: ?Sized + NixPath>(path: &P) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            stat64(cstr.as_ptr(), &mut dst as *mut FileStat)
        }
    }));

    try!(Errno::result(res));

    Ok(dst)
}

pub fn lstat<P: ?Sized + NixPath>(path: &P) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            lstat64(cstr.as_ptr(), &mut dst as *mut FileStat)
        }
    }));

    try!(Errno::result(res));

    Ok(dst)
}

pub fn fstat(fd: RawFd) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = unsafe { fstat64(fd, &mut dst as *mut FileStat) };

    try!(Errno::result(res));

    Ok(dst)
}

pub fn fstatat<P: ?Sized + NixPath>(dirfd: RawFd, pathname: &P, f: AtFlags) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = try!(pathname.with_nix_path(|cstr| {
        unsafe { fstatat64(dirfd, cstr.as_ptr(), &mut dst as *mut FileStat, f.bits() as libc::c_int) }
    }));

    try!(Errno::result(res));

    Ok(dst)
}

/// Change the file permission bits of the file specified by a file descriptor.
///
/// # References
///
/// [fchmod(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/fchmod.html).
pub fn fchmod(fd: RawFd, mode: Mode) -> Result<()> {
    let res = unsafe { libc::fchmod(fd, mode.bits() as mode_t) };

    Errno::result(res).map(|_| ())
}

/// Flags for `fchmodat` function.
#[derive(Clone, Copy, Debug)]
pub enum FchmodatFlags {
    FollowSymlink,
    NoFollowSymlink,
}

/// Change the file permission bits.
///
/// The file to be changed is determined relative to the directory associated
/// with the file descriptor `dirfd` or the current working directory
/// if `dirfd` is `None`.
///
/// If `flag` is `FchmodatFlags::NoFollowSymlink` and `path` names a symbolic link,
/// then the mode of the symbolic link is changed.
///
/// `fchmod(None, path, mode, FchmodatFlags::FollowSymlink)` is identical to
/// a call `libc::chmod(path, mode)`. That's why `chmod` is unimplemented
/// in the `nix` crate.
///
/// # References
///
/// [fchmodat(2)](http://pubs.opengroup.org/onlinepubs/9699919799/functions/fchmodat.html).
pub fn fchmodat<P: ?Sized + NixPath>(
    dirfd: Option<RawFd>,
    path: &P,
    mode: Mode,
    flag: FchmodatFlags,
) -> Result<()> {
    let actual_dirfd =
        match dirfd {
            None => libc::AT_FDCWD,
            Some(fd) => fd,
        };
    let atflag =
        match flag {
            FchmodatFlags::FollowSymlink => AtFlags::empty(),
            FchmodatFlags::NoFollowSymlink => AtFlags::AT_SYMLINK_NOFOLLOW,
        };
    let res = path.with_nix_path(|cstr| unsafe {
        libc::fchmodat(
            actual_dirfd,
            cstr.as_ptr(),
            mode.bits() as mode_t,
            atflag.bits() as libc::c_int,
        )
    })?;

    Errno::result(res).map(|_| ())
}

#[cfg(target_os = "linux")]
mod linux {
    use {Errno, Result, NixPath};
    use std::os::unix::io::RawFd;
    use libc;
    use fcntl::AtFlags;
    use sys::time::TimeSpec;

    /// A file timestamp.
    #[derive(Clone, Copy, Debug)]
    pub enum UtimeSpec {
        /// File timestamp is set to the current time.
        Now,
        /// The corresponding file timestamp is left unchanged.
        Omit,
        /// File timestamp is set to value
        Time(TimeSpec)
    }

    impl <'a> From<&'a UtimeSpec> for libc::timespec {
        fn from(time: &'a UtimeSpec) -> libc::timespec { 
            match time {
                &UtimeSpec::Now => libc::timespec {
                    tv_sec: 0,
                    tv_nsec: libc::UTIME_NOW,
                },
                &UtimeSpec::Omit => libc::timespec {
                    tv_sec: 0,
                    tv_nsec: libc::UTIME_OMIT,
                },
                &UtimeSpec::Time(spec) => *spec.as_ref()
            }
        }
    }

    /// Change file timestamps with nanosecond precision
    /// (see [utimensat(2)](http://man7.org/linux/man-pages/man2/utimensat.2.html)).
    pub fn utimensat<P: ?Sized + NixPath>(dirfd: RawFd,
                                          pathname: &P,
                                          atime: &UtimeSpec,
                                          mtime: &UtimeSpec,
                                          flags: AtFlags) -> Result<()> {
        let time = [atime.into(), mtime.into()];
        let res = try!(pathname.with_nix_path(|cstr| {
            unsafe {
                libc::utimensat(dirfd,
                                cstr.as_ptr(),
                                time.as_ptr() as *const libc::timespec,
                                flags.bits())
            }
        }));

        Errno::result(res).map(drop)
    }

    /// Change file timestamps with nanosecond precision
    /// (see [futimens(2)](http://man7.org/linux/man-pages/man2/futimens.2.html)).
    pub fn futimens(fd: RawFd,
                    atime: &UtimeSpec,
                    mtime: &UtimeSpec) -> Result<()> {
        let time = [atime.into(), mtime.into()];
        let res = unsafe {
            libc::futimens(fd, time.as_ptr() as *const libc::timespec)
        };
    
        Errno::result(res).map(drop)
    }
}

#[cfg(not(target_os = "linux"))]
mod linux { }
