pub use libc::rlimit64 as Rlimit;

use libc;
use std::mem;

libc_enum!{
    #[repr(i32)]
    pub enum Resource {
        RLIMIT_AS,
        RLIMIT_CORE,
        RLIMIT_CPU,
        RLIMIT_DATA,
        RLIMIT_FSIZE,
        RLIMIT_NOFILE,
        RLIMIT_STACK,
        // TODO
        //#[cfg(any(target_os = "android", target_os = "linux"))]
        //RLIMIT_LOCKS,
        //RLIMIT_MEMLOCK,
        //RLIMIT_MSGQUEUE,
        //RLIMIT_NICE,
        //RLIMIT_NLIMITS,
        //RLIMIT_NOVMON,
        //RLIMIT_NPROC,
        //RLIMIT_NPTS,
        //_RLIMIT_POSIX_FLAG,
        //RLIMIT_POSIXLOCKS,
        //RLIMIT_RSS,
        //RLIMIT_RTPRIO:
        //RLIMIT_RTTIME,
        //RLIMIT_SBSIZE,
        //RLIMIT_SIGPENDING,
        //RLIMIT_SWAP,
        //RLIMIT_VMEM,
    }
}


use {Errno, Result};

pub fn getrlimit(resource: Resource) -> Result<Rlimit> {
    let mut rlimit = unsafe { mem::uninitialized() };
    let res = unsafe {
        libc::getrlimit64(resource as libc::c_int, &mut rlimit as *mut Rlimit)

    };
    Errno::result(res).map(|_| rlimit)
}

pub fn setrlimit(resource: Resource, rlimit: &Rlimit) -> Result<()> {
    let res = unsafe {
        libc::setrlimit64(resource as libc::c_int, rlimit as *const Rlimit)
    };
    Errno::result(res).map(drop)
}
