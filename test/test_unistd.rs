extern crate tempdir;

use nix::unistd::*;
use nix::unistd::ForkResult::*;
use nix::sys::wait::*;
use nix::sys::stat;
use nix::fcntl;
use std::iter;
use std::ffi::CString;
use std::fs::File;
use std::io::{Write, Read};
use std::os::unix::prelude::*;
use std::env::current_dir;
use tempfile::tempfile;
use tempdir::TempDir;
use libc::off_t;

#[test]
fn test_fork_and_waitpid() {
    let pid = fork();
    match pid {
        Ok(Child) => {} // ignore child here
        Ok(Parent { child }) => {
            // assert that child was created and pid > 0
            assert!(child > 0);
            let wait_status = waitpid(child, None);
            match wait_status {
                // assert that waitpid returned correct status and the pid is the one of the child
                Ok(WaitStatus::Exited(pid_t, _)) =>  assert!(pid_t == child),

                // panic, must never happen
                Ok(_) => panic!("Child still alive, should never happen"),

                // panic, waitpid should never fail
                Err(_) => panic!("Error: waitpid Failed")
            }

        },
        // panic, fork should never fail unless there is a serious problem with the OS
        Err(_) => panic!("Error: Fork Failed")
    }
}

#[test]
fn test_wait() {
    let pid = fork();
    match pid {
        Ok(Child) => {} // ignore child here
        Ok(Parent { child }) => {
            let wait_status = wait();

            // just assert that (any) one child returns with WaitStatus::Exited
            assert_eq!(wait_status, Ok(WaitStatus::Exited(child, 0)));
        },
        // panic, fork should never fail unless there is a serious problem with the OS
        Err(_) => panic!("Error: Fork Failed")
    }
}

#[test]
fn test_mkstemp() {
    let result = mkstemp("/tmp/nix_tempfile.XXXXXX");
    match result {
        Ok((fd, path)) => {
            close(fd).unwrap();
            unlink(path.as_path()).unwrap();
        },
        Err(e) => panic!("mkstemp failed: {}", e)
    }

    let result = mkstemp("/tmp/");
    match result {
        Ok(_) => {
            panic!("mkstemp succeeded even though it should fail (provided a directory)");
        },
        Err(_) => {}
    }
}

#[test]
fn test_getpid() {
    let pid = getpid();
    let ppid = getppid();
    assert!(pid > 0);
    assert!(ppid > 0);
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux_android {
    use nix::unistd::gettid;

    #[test]
    fn test_gettid() {
        let tid = gettid();
        assert!(tid > 0);
    }

}
#[test]
fn test_mkdirat() {
    let tempdir = TempDir::new("nix-test_mkdirat").unwrap();
    let path = tempdir.path().join("test_path");

    let dirfd = fcntl::open(tempdir.path(),
                            fcntl::OFlag::empty(),
                            stat::Mode::empty());

    mkdirat(dirfd.unwrap(),
            &path.file_name(),
            stat::Mode::empty()).unwrap();
    assert!(path.exists());
}

#[test]
fn test_access() {
    let tempdir = TempDir::new("nix-test_mkdirat").unwrap();

    let dirfd = fcntl::open(tempdir.path().parent().unwrap(),
                            fcntl::OFlag::empty(),
                            stat::Mode::empty());

    // if succeed, permissions are or ok
    access(tempdir.path(), R_OK | X_OK | W_OK).unwrap();

    faccessat(dirfd.unwrap(),
              &tempdir.path().file_name(),
              R_OK | X_OK | W_OK,
              fcntl::AtFlags::empty()).unwrap();

}

macro_rules! execve_test_factory(
    ($test_name:ident, $syscall:ident, $unix_sh:expr, $android_sh:expr) => (
    #[test]
    fn $test_name() {
        // The `exec`d process will write to `writer`, and we'll read that
        // data from `reader`.
        let (reader, writer) = pipe().unwrap();

        match fork().unwrap() {
            Child => {
                #[cfg(not(target_os = "android"))]
                const SH_PATH: &'static [u8] = $unix_sh;

                #[cfg(target_os = "android")]
                const SH_PATH: &'static [u8] = $android_sh;

                // Close stdout.
                close(1).unwrap();
                // Make `writer` be the stdout of the new process.
                dup(writer).unwrap();
                // exec!
                $syscall(
                    &CString::new(SH_PATH).unwrap(),
                    &[CString::new(b"".as_ref()).unwrap(),
                      CString::new(b"-c".as_ref()).unwrap(),
                      CString::new(b"echo nix!!! && echo foo=$foo && echo baz=$baz"
                                   .as_ref()).unwrap()],
                    &[CString::new(b"foo=bar".as_ref()).unwrap(),
                      CString::new(b"baz=quux".as_ref()).unwrap()]).unwrap();
            },
            Parent { child } => {
                // Wait for the child to exit.
                waitpid(child, None).unwrap();
                // Read 1024 bytes.
                let mut buf = [0u8; 1024];
                read(reader, &mut buf).unwrap();
                // It should contain the things we printed using `/bin/sh`.
                let string = String::from_utf8_lossy(&buf);
                assert!(string.contains("nix!!!"));
                assert!(string.contains("foo=bar"));
                assert!(string.contains("baz=quux"));
            }
        }
    }
    )
);

#[test]
fn test_fchdir() {
    let tmpdir = TempDir::new("test_fchdir").unwrap();
    let tmpdir_path = tmpdir.path().canonicalize().unwrap();
    let tmpdir_fd = File::open(&tmpdir_path).unwrap().into_raw_fd();
    let olddir_path = getcwd().unwrap();
    let olddir_fd = File::open(&olddir_path).unwrap().into_raw_fd();

    assert!(fchdir(tmpdir_fd).is_ok());
    assert_eq!(getcwd().unwrap(), tmpdir_path);

    assert!(fchdir(olddir_fd).is_ok());
    assert_eq!(getcwd().unwrap(), olddir_path);

    assert!(close(olddir_fd).is_ok());
    assert!(close(tmpdir_fd).is_ok());
}

#[test]
fn test_getcwd() {
    let tmp_dir = TempDir::new("test_getcwd").unwrap();
    assert!(chdir(tmp_dir.path()).is_ok());
    assert_eq!(getcwd().unwrap(), current_dir().unwrap());

    // make path 500 chars longer so that buffer doubling in getcwd kicks in.
    // Note: One path cannot be longer than 255 bytes (NAME_MAX)
    // whole path cannot be longer than PATH_MAX (usually 4096 on linux, 1024 on macos)
    let mut inner_tmp_dir = tmp_dir.path().to_path_buf();
    for _ in 0..5 {
        let newdir = iter::repeat("a").take(100).collect::<String>();
        //inner_tmp_dir = inner_tmp_dir.join(newdir).path();
        inner_tmp_dir.push(newdir);
        assert!(mkdir(inner_tmp_dir.as_path(), stat::S_IRWXU).is_ok());
    }
    assert!(chdir(inner_tmp_dir.as_path()).is_ok());
    assert_eq!(getcwd().unwrap(), current_dir().unwrap());
}

#[test]
fn test_lseek() {
    const CONTENTS: &'static [u8] = b"abcdef123456";
    let mut tmp = tempfile().unwrap();
    tmp.write(CONTENTS).unwrap();

    let offset: off_t = 5;
    lseek(tmp.as_raw_fd(), offset, Whence::SeekSet).unwrap();

    let mut buf = String::new();
    tmp.read_to_string(&mut buf).unwrap();
    assert_eq!(b"f123456", buf.as_bytes());

    close(tmp.as_raw_fd()).unwrap();
}

#[test]
fn test_unlinkat() {
    let tempdir = TempDir::new("nix-test_unlinkat").unwrap();
    let dirfd = fcntl::open(tempdir.path(),
                            fcntl::OFlag::empty(),
                            stat::Mode::empty());
    let file = tempdir.path().join("foo");
    File::create(&file).unwrap();

    unlinkat(dirfd.unwrap(),
            &file.file_name(),
            fcntl::AtFlags::empty()).unwrap();
    assert!(!file.exists());
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[test]
fn test_lseek64() {
    const CONTENTS: &'static [u8] = b"abcdef123456";
    let mut tmp = tempfile().unwrap();
    tmp.write(CONTENTS).unwrap();

    lseek64(tmp.as_raw_fd(), 5, Whence::SeekSet).unwrap();

    let mut buf = String::new();
    tmp.read_to_string(&mut buf).unwrap();
    assert_eq!(b"f123456", buf.as_bytes());

    close(tmp.as_raw_fd()).unwrap();
}

#[test]
fn test_linkat() {
    let tempdir = TempDir::new("nix-test_linkat").unwrap();
    let src = tempdir.path().join("foo");
    let dst = tempdir.path().join("bar");
    File::create(&src).unwrap();

    let dirfd = fcntl::open(tempdir.path(),
                            fcntl::OFlag::empty(),
                            stat::Mode::empty());
    linkat(dirfd.unwrap(),
           &src.file_name(),
           dirfd.unwrap(),
           &dst.file_name(),
           fcntl::AtFlags::empty()).unwrap();
    assert!(dst.exists());
}

#[test]
fn test_link() {
    let tempdir = TempDir::new("nix-test_link").unwrap();
    let src = tempdir.path().join("foo");
    let dst = tempdir.path().join("bar");
    File::create(&src).unwrap();

    link(&src, &dst).unwrap();
    assert!(dst.exists());
}

execve_test_factory!(test_execve, execve, b"/bin/sh", b"/system/bin/sh");

#[cfg(any(target_os = "linux", target_os = "android"))]
#[cfg(feature = "execvpe")]
execve_test_factory!(test_execvpe, execvpe, b"sh", b"sh");
