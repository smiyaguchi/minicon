use nix::fcntl::OFlag;
use nix::unistd::{pipe2, read, close};
use std::os::unix::io::RawFd;
use super::Result;

pub struct Pipe {
    rfd: RawFd,
    wfd: RawFd,
}

impl Pipe {
    pub fn new() -> Result<Pipe> {
        let (rfd, wfd) = pipe2(OFlag::O_CLOEXEC)?;
        Ok(Pipe { rfd: rfd, wfd: wfd })
    }

    pub fn wait(&self) -> Result<()> {
        close(self.wfd)?;
        let data: &mut [u8] = &mut [0];
        while read(self.rfd, data)? != 0 {}
        close(self.rfd)?;
        Ok(())
    }

    pub fn notify(&self) -> Result<()> {
        close(self.rfd)?;
        close(self.wfd)?;
        Ok(())
    }
}
