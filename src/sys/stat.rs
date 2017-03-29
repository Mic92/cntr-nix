pub use libc::dev_t;
pub use libc::stat as FileStat;

use {Errno, Result, NixPath};
use fcntl::AtFlags;
use libc::{self, mode_t};
use std::mem;
use std::os::unix::io::RawFd;

libc_bitflags!(
    pub flags SFlag: mode_t {
        S_IFIFO,
        S_IFCHR,
        S_IFDIR,
        S_IFBLK,
        S_IFREG,
        S_IFLNK,
        S_IFSOCK,
        S_IFMT,
    }
);

bitflags! {
    pub flags Mode: mode_t {
        const S_IRWXU = libc::S_IRWXU,
        const S_IRUSR = libc::S_IRUSR,
        const S_IWUSR = libc::S_IWUSR,
        const S_IXUSR = libc::S_IXUSR,

        const S_IRWXG = libc::S_IRWXG,
        const S_IRGRP = libc::S_IRGRP,
        const S_IWGRP = libc::S_IWGRP,
        const S_IXGRP = libc::S_IXGRP,

        const S_IRWXO = libc::S_IRWXO,
        const S_IROTH = libc::S_IROTH,
        const S_IWOTH = libc::S_IWOTH,
        const S_IXOTH = libc::S_IXOTH,

        const S_ISUID = libc::S_ISUID as mode_t,
        const S_ISGID = libc::S_ISGID as mode_t,
        const S_ISVTX = libc::S_ISVTX as mode_t,
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

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub fn mknodat<P: ?Sized + NixPath>(dirfd: RawFd, path: &P, kind: SFlag, perm: Mode, dev: dev_t) -> Result<()> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            libc::mknodat(dirfd, cstr.as_ptr(), kind.bits | perm.bits() as mode_t, dev)
        }
    }));

    Errno::result(res).map(drop)
}

#[cfg(target_os = "linux")]
pub fn major(dev: dev_t) -> u64 {
    ((dev >> 32) & 0xfffff000) |
    ((dev >>  8) & 0x00000fff)
}

#[cfg(target_os = "linux")]
pub fn minor(dev: dev_t) -> u64 {
    ((dev >> 12) & 0xffffff00) |
    ((dev      ) & 0x000000ff)
}

#[cfg(target_os = "linux")]
pub fn makedev(major: u64, minor: u64) -> dev_t {
    ((major & 0xfffff000) << 32) |
    ((major & 0x00000fff) <<  8) |
    ((minor & 0xffffff00) << 12) |
    ((minor & 0x000000ff)      )
}

pub fn umask(mode: Mode) -> Mode {
    let prev = unsafe { libc::umask(mode.bits() as mode_t) };
    Mode::from_bits(prev).expect("[BUG] umask returned invalid Mode")
}

pub fn stat<P: ?Sized + NixPath>(path: &P) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            libc::stat(cstr.as_ptr(), &mut dst as *mut FileStat)
        }
    }));

    Errno::result(res).map(|_| dst)
}

pub fn lstat<P: ?Sized + NixPath>(path: &P) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            libc::lstat(cstr.as_ptr(), &mut dst as *mut FileStat)
        }
    }));

    Errno::result(res).map(|_| dst)
}

pub fn fstat(fd: RawFd) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = unsafe { libc::fstat(fd, &mut dst as *mut FileStat) };

    Errno::result(res).map(|_| dst)
}

pub fn fstatat<P: ?Sized + NixPath>(dirfd: RawFd, pathname: &P, flags: AtFlags) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = try!(pathname.with_nix_path(|cstr| {
        unsafe { libc::fstatat(dirfd, cstr.as_ptr(), &mut dst as *mut FileStat, flags.bits()) }
    }));

    Errno::result(res).map(|_| dst)
}

pub fn chmod<P: ?Sized + NixPath>(pathname: &P, mode: Mode) -> Result<()> {
    let res = try!(pathname.with_nix_path(|cstr| {
        unsafe { libc::chmod(cstr.as_ptr(), mode.bits()) }
    }));

    Errno::result(res).map(drop)
}

pub fn fchmod(fd: RawFd, mode: Mode) -> Result<()> {
    let res = unsafe { libc::fchmod(fd, mode.bits()) };

    Errno::result(res).map(drop)
}

pub fn fchmodat<P: ?Sized + NixPath>(dirfd: RawFd, pathname: &P, mode: Mode, flags: AtFlags) -> Result<()> {
    let res = try!(pathname.with_nix_path(|cstr| {
        unsafe {
            libc::fchmodat(dirfd,
                           cstr.as_ptr(),
                           mode.bits(),
                           flags.bits())
        }
    }));

    Errno::result(res).map(drop)
}
