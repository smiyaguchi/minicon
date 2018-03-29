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

#[inline]
pub fn setrlimit(
    resource: libc::c_int,
    soft: libc::c_ulonglong,
    hard: libc::c_ulonglong,
) -> Result<()> {
    let rlim = &libc::rlimit {
        rlim_cur: soft,
        rlim_max: hard,
    };
    let res = unsafe { libc::setrlimit(resource, rlim) };
    Errno::result(res).map(drop)
}
