use libc::{self, c_int};
use std::os::unix::io::RawFd;
use {Result, NixPath};
use errno::Errno;

#[allow(unused)]
fn getxattr<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(
    path: &P1,
    name: &P2,
    buf: &mut [u8],
    ) -> Result<usize> {
    let res = try!(try!(unsafe {
        path.with_nix_path(|p| {
            name.with_nix_path(|n| {
                libc::getxattr(
                    p.as_ptr(),
                    n.as_ptr(),
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                    )
            })
        })
    }));
    Errno::result(res).map(|size| size as usize)
}

#[allow(unused)]
fn fgetxattr<P: ?Sized + NixPath>(fd: RawFd, name: &P, buf: &mut [u8]) -> Result<usize> {
    let res = try!(unsafe {
        name.with_nix_path(|cstr| {
            libc::fgetxattr(
                fd,
                cstr.as_ptr(),
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                )
        })
    });
    Errno::result(res).map(|size| size as usize)
}

#[allow(unused)]
fn lgetxattr<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(
    path: &P1,
    name: &P2,
    buf: &mut [u8],
    ) -> Result<usize> {
    let res = try!(try!(unsafe {
        path.with_nix_path(|p| {
            name.with_nix_path(|n| {
                libc::lgetxattr(
                    p.as_ptr(),
                    n.as_ptr(),
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                    )
            })
        })
    }));
    Errno::result(res).map(|size| size as usize)
}

#[allow(unused)]
fn listxattr<P: ?Sized + NixPath>(path: &P, list: &mut [u8]) -> Result<usize> {
    let res = try!(unsafe {
        path.with_nix_path(|cstr| {
            libc::listxattr(cstr.as_ptr(), list.as_mut_ptr() as *mut i8, list.len())
        })
    });
    Errno::result(res).map(|size| size as usize)
}

#[allow(unused)]
fn flistxattr(fd: RawFd, list: &mut [u8]) -> Result<usize> {
    let res = unsafe { libc::flistxattr(fd, list.as_mut_ptr() as *mut i8, list.len()) };
    Errno::result(res).map(|size| size as usize)
}

#[allow(unused)]
fn llistxattr<P: ?Sized + NixPath>(path: &P, list: &mut [u8]) -> Result<usize> {
    let res = try!(unsafe {
        path.with_nix_path(|cstr| {
            libc::llistxattr(cstr.as_ptr(), list.as_mut_ptr() as *mut i8, list.len())
        })
    });
    Errno::result(res).map(|size| size as usize)
}

#[allow(unused)]
fn lsetxattr<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(
    path: &P1,
    name: &P2,
    buf: &[u8],
    flags: c_int,
    ) -> Result<()> {
    let res = try!(try!(unsafe {
        path.with_nix_path(|p| {
            name.with_nix_path(|n| {
                libc::lsetxattr(
                    p.as_ptr(),
                    n.as_ptr(),
                    buf.as_ptr() as *const libc::c_void,
                    buf.len(),
                    flags,
                    )
            })
        })
    }));
    Errno::result(res).map(drop)
}

#[allow(unused)]
fn setxattr<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(
    path: &P1,
    name: &P2,
    buf: &[u8],
    flags: c_int,
    ) -> Result<()> {
    let res = try!(try!(unsafe {
        path.with_nix_path(|p| {
            name.with_nix_path(|n| {
                libc::setxattr(
                    p.as_ptr(),
                    n.as_ptr(),
                    buf.as_ptr() as *const libc::c_void,
                    buf.len(),
                    flags,
                    )
            })
        })
    }));
    Errno::result(res).map(drop)
}

#[allow(unused)]
fn fsetxattr<P: ?Sized + NixPath>(fd: RawFd, name: &P, buf: &[u8], flags: c_int) -> Result<()> {
    let res = try!(unsafe {
        name.with_nix_path(|cstr| {
            libc::fsetxattr(
                fd,
                cstr.as_ptr(),
                buf.as_ptr() as *const libc::c_void,
                buf.len(),
                flags)
        })
    });
    Errno::result(res).map(drop)
}

#[allow(unused)]
fn removexattr<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(path: &P1, name: &P2) -> Result<()> {
    let res = try!(try!(unsafe {
        path.with_nix_path(|p| {
            name.with_nix_path(|n| libc::removexattr(p.as_ptr(), n.as_ptr()))
        })
    }));
    Errno::result(res).map(drop)
}

#[allow(unused)]
fn fremovexattr<P: ?Sized + NixPath>(fd: RawFd, name: &P) -> Result<()> {
    let res = try!(unsafe {
        name.with_nix_path(|cstr| libc::fremovexattr(fd, cstr.as_ptr()))
    });
    Errno::result(res).map(drop)
}

#[allow(unused)]
fn lremovexattr<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(path: &P1, name: &P2) -> Result<()> {
    let res = try!(try!(unsafe {
        path.with_nix_path(|p| {
            name.with_nix_path(|n| libc::lremovexattr(p.as_ptr(), n.as_ptr()))
        })
    }));
    Errno::result(res).map(drop)
}
