// Copyright 2026 https://github.com/KernelSU-Modules-Repo/meta-overlayfs and https://github.com/bmax121/APatch

#[cfg(any(target_os = "linux", target_os = "android"))]
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
use std::{io::Read, os::fd::AsFd};

#[cfg(any(target_os = "linux", target_os = "android"))]
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
#[cfg(any(target_os = "linux", target_os = "android"))]
use loopdev::LoopControl;
#[cfg(any(target_os = "linux", target_os = "android"))]
use rustix::mount::{MountFlags, UnmountFlags, mount, unmount};
use rustix::{
    fs::CWD,
    mount::{
        FsMountFlags, FsOpenFlags, MountAttrFlags, MoveMountFlags, fsconfig_create,
        fsconfig_set_string, fsmount, fsopen, move_mount,
    },
};

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn mount_ext4<P>(source: P, target: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = source.as_ref();
    if !path.exists() {
        println!("Source path does not exist");
    } else {
        let metadata = fs::metadata(path)?;
        let permissions = metadata.permissions();
        let mode = permissions.mode();

        if permissions.readonly() {
            println!("File permissions: {:o} (octal)", mode & 0o777);
        }
    }

    mount_ext4_loop(source.as_ref(), target.as_ref())?;
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn mount_ext4_loop<P>(source: P, target: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let lc = LoopControl::open().context("Failed to open loop control")?;
    let ld = lc.next_free().context("Failed to find free loop device")?;

    ld.with()
        .read_only(false)
        .autoclear(true)
        .attach(source.as_ref())
        .context("Failed to attach source to loop device")?;

    let device_path = ld.path().context("Could not get loop device path")?;
    log::debug!("loop device path: {}", device_path.display());

    mount(
        &device_path,
        target.as_ref(),
        "ext4",
        MountFlags::NOATIME,
        Some(c""),
    )
    .context(format!(
        "Failed to mount {} to {}",
        device_path.display(),
        target.as_ref().display()
    ))?;

    Ok(())
}

pub fn is_overlay_supported() -> Result<bool> {
    let file = fs::File::open("/proc/config.gz")?;

    let mut config = String::new();
    let mut decoder = GzDecoder::new(file);
    decoder.read_to_string(&mut config)?;

    for i in config.lines() {
        if i.starts_with("#") {
            continue;
        }

        let Some((k, v)) = i.split_once('=') else {
            continue;
        };

        if k.trim() == "CONFIG_OVERLAY_FS" && v.trim() == "y" {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn umount_dir(src: impl AsRef<Path>) -> Result<()> {
    unmount(src.as_ref(), UnmountFlags::empty())
        .with_context(|| format!("Failed to umount {}", src.as_ref().display()))?;
    Ok(())
}

pub fn fs<S, P>(
    upperdir: Option<String>,
    workdir: Option<String>,
    lowerdir_config: String,
    source: S,
    dest: P,
) -> Result<()>
where
    S: ToString,
    P: AsRef<Path>,
{
    let fs = fsopen("overlay", FsOpenFlags::FSOPEN_CLOEXEC).context("Failed to fsopen overlay")?;
    let fs = fs.as_fd();
    fsconfig_set_string(fs, "lowerdir", &lowerdir_config)
        .context("Failed to fsconfig set string lowerdir with {lowerdir_config}")?;
    if let (Some(upperdir), Some(workdir)) = (&upperdir, &workdir) {
        fsconfig_set_string(fs, "upperdir", upperdir)
            .context("Failed to fsconfig set string upperdir with {upperdir}")?;
        fsconfig_set_string(fs, "workdir", workdir)
            .context("Failed to fsconfig set string workdir with {workdir}")?;
    }
    fsconfig_set_string(fs, "source", source.to_string())
        .context("Failed to fsconfig set string source with {source}")?;
    fsconfig_create(fs).context("Failed to fsconfig create new fs")?;
    let mount = fsmount(fs, FsMountFlags::FSMOUNT_CLOEXEC, MountAttrFlags::empty())
        .context("Failed to mount")?;
    move_mount(
        mount.as_fd(),
        "",
        CWD,
        dest.as_ref(),
        MoveMountFlags::MOVE_MOUNT_F_EMPTY_PATH,
    )?;

    Ok(())
}
