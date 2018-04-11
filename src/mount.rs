use errors::*;
use nix::fcntl::open;
use nix::fcntl::OFlag;
use nix::mount::umount2;
use nix::mount::MntFlags;
use nix::NixPath;
use nix::unistd::{pivot_root, fchdir};
use nix::sys::stat::Mode;

pub fn do_pivot_root<P: ?Sized + NixPath>(path: &P) -> Result<()> {
    let old_root = open("/", OFlag::O_DIRECTORY | OFlag::O_RDONLY, Mode::empty())?;
    let new_root = open(path, OFlag::O_DIRECTORY | OFlag::O_RDONLY, Mode::empty())?;
    pivot_root(path, path)?;
    umount2("/", MntFlags::MNT_DETACH)?;
    fchdir(new_root)?;
    Ok(())
}
