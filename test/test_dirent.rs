use nix::dirent::{self, opendir, readdir, seekdir, telldir, Dir};
use std::path::Path;
use tempdir::TempDir;

fn test_readdir<OPEN>(open_fn: OPEN)
    where OPEN: Fn(&Path) -> Dir
{
    let tempdir = TempDir::new("nix-test_readdir")
        .unwrap_or_else(|e| panic!("tempdir failed: {}", e));
    let mut dir = open_fn(tempdir.path());
    let letter1 = readdir(&mut dir)
                       .unwrap()
                       .unwrap()
                       .name()
                       .to_bytes()[0];
    assert_eq!(letter1, '.' as u8);

    let pos = telldir(&mut dir);
    seekdir(&mut dir, pos); // no-op

    let letter2 = readdir(&mut dir)
                       .unwrap()
                       .unwrap()
                       .name()
                       .to_bytes()[0];
    assert_eq!(letter2, '.' as u8);

    assert!(readdir(&mut dir).unwrap().is_none());
}

#[test]
fn test_opendir() {
    test_readdir(|path| opendir(path).unwrap());
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[test]
fn test_fdopendir() {
    use std::os::unix::io::IntoRawFd;
    use std::fs::File;
    test_readdir(|path| {
        let fd = File::open(path).unwrap().into_raw_fd();
        let mut dirp = dirent::fdopendir(fd).unwrap();
        assert_eq!(fd, dirent::dirfd(&mut dirp).unwrap());
        dirp
    });
}
