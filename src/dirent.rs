use {Result, Error, Errno, NixPath};
use errno;
use libc::{self, DIR, c_long};
use std::convert::{AsRef, Into};
use std::ffi::CStr;
use std::mem;

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
use std::os::unix::io::RawFd;

pub struct Dir(*mut DIR);

impl AsRef<DIR> for Dir {
    fn as_ref(&self) -> &DIR {
        unsafe { &*self.0 }
    }
}

impl Into<*mut DIR> for Dir {
    fn into(self) -> *mut DIR {
        let dirp = self.0;
        mem::forget(self);
        dirp
    }
}

impl Into<Dir> for *mut DIR {
    fn into(self) -> Dir {
        Dir(self)
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        unsafe { libc::closedir(self.0) };
    }
}

pub struct Dirent<'a>(&'a libc::dirent);

impl<'a> Dirent<'a> {
    pub fn name(&self) -> &CStr {
        unsafe{
            CStr::from_ptr(self.0.d_name.as_ptr())
        }
    }

    pub fn ino(&self) -> libc::ino_t {
        self.0.d_ino
    }
}

impl<'a> AsRef<libc::dirent> for Dirent<'a> {
    fn as_ref(&self) -> &libc::dirent {
        self.0
    }
}

pub fn opendir<P: ?Sized + NixPath>(name: &P) -> Result<Dir> {
    let dirp = try!(name.with_nix_path(|cstr| unsafe { libc::opendir(cstr.as_ptr()) }));
    if dirp.is_null() {
        Err(Error::last().into())
    } else {
        Ok(Dir(dirp))
    }
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub fn fdopendir(fd: RawFd) -> Result<Dir> {
    let dirp = unsafe { libc::fdopendir(fd) };
    if dirp.is_null() {
        Err(Error::last().into())
    } else {
        Ok(Dir(dirp))
    }
}

pub fn readdir<'a>(dir: &'a mut Dir) -> Result<Option<Dirent>> {
    let dirent = unsafe {
        Errno::clear();
        libc::readdir(dir.0)
    };
    if dirent.is_null() {
        match Errno::last() {
            errno::UnknownErrno => Ok(None),
            _ => Err(Error::last().into()),
        }
    } else {
        Ok(Some(Dirent(unsafe { &*dirent })))
    }
}

pub fn seekdir<'a>(dir: &'a mut Dir, loc: c_long) {
    unsafe { libc::seekdir(dir.0, loc) };
}

pub fn telldir<'a>(dir: &'a Dir) -> c_long {
    unsafe { libc::telldir(dir.0) }
}

pub fn dirfd<'a>(dir: &'a Dir) -> Result<RawFd> {
    let res = unsafe { libc::dirfd(dir.0) };
    Errno::result(res)
}
