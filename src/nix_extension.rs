use libc;
use nix::errno::Errno;
use nix::Result;

#[inline]
pub fn clearenv() -> Result<()> {
    let res = unsafe { libc::clearenv() };
    Errno::result(res).map(drop)  
}
