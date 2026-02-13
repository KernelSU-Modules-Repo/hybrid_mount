use std::{
    collections::HashSet,
    path::Path,
    sync::{LazyLock, Mutex, OnceLock},
};

use anyhow::Result;
use ksu::{TryUmount, TryUmountFlags};
use rustix::path::Arg;

pub static TMPFS: OnceLock<String> = OnceLock::new();
pub static LIST: LazyLock<Mutex<TryUmount>> = LazyLock::new(|| Mutex::new(TryUmount::new()));
static HISTORY: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

const DONNOT_UMOUNT_LIST: &[&str] = &[
    "/vendor/lib",
    "/vendor/lib64",
    "/system/lib",
    "/system/lib64",
];

pub fn send_umountable<P>(target: P) -> Result<()>
where
    P: AsRef<Path>,
{
    if !crate::utils::KSU.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }

    let target = target.as_ref();
    if DONNOT_UMOUNT_LIST.iter().any(|s| target.starts_with(s)) {
        log::warn!("Cannot umount paths starting with /vendor/lib!!");
        return Ok(());
    }

    let path = target.as_str()?;
    let mut history = HISTORY
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to lock history mutex"))?;

    for i in history.iter() {
        if i.starts_with(path) {
            log::debug!("umount list already includes the parent directory of {path}.");
            return Ok(());
        }
    }

    history.insert(path.to_string());
    LIST.lock()
        .map_err(|_| anyhow::anyhow!("Failed to lock umount list"))?
        .add(target);
    Ok(())
}

pub fn commit() -> Result<()> {
    if !crate::utils::KSU.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }
    let mut list = LIST
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to lock umount list"))?;

    list.format_msg(|p| format!("{p:?} umount successful "));
    list.flags(TryUmountFlags::MNT_FORCE);
    if let Err(e0) = list.umount() {
        log::debug!("try_umount(0) failed: {:#}, retrying with flags(2)", e0);

        list.flags(TryUmountFlags::MNT_DETACH);
        if let Err(e2) = list.umount() {
            log::warn!("try_umount(2) failed: {:#}", e2);
        }
    }

    Ok(())
}
