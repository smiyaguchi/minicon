use libc;
use nix::errno::Errno;
use nix::Result;
use std::ffi::CString;

#[inline]
pub fn clearenv() -> Result<()> {
    let res = unsafe { libc::clearenv() };
    Errno::result(res).map(drop)  
}

#[cfg(target_env = "gnu")]
#[inline]
pub fn putenv(string: &CString) -> Result<()> {
    let ptr = string.clone().into_raw();
    let res = unsafe { libc::putenv(ptr as *mut libc::c_char) };
    Errno::result(res).map(drop)  
}
