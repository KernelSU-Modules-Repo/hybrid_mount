use std::collections::HashSet;
use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use log::{debug, info, warn};
use procfs::process::Process;
use rustix::{
    fd::AsFd,
    fs::CWD,
    mount::{
        MountFlags, MoveMountFlags, OpenTreeFlags, UnmountFlags, mount, move_mount, open_tree,
        unmount,
    },
};

use crate::defs::KSU_OVERLAY_SOURCE;

fn mount_overlayfs(
    lower_dirs: &[String],
    lowest: &str,
    upperdir: Option<&Path>,
    workdir: Option<&Path>,
    dest: &Path,
) -> Result<()> {
    let lowerdir_config = lower_dirs
        .iter()
        .map(|s| s.as_str())
        .chain(std::iter::once(lowest))
        .collect::<Vec<_>>()
        .join(":");

    debug!(
        "mount overlayfs on {:?}, lowerdir={}, upperdir={:?}, workdir={:?}",
        dest, lowerdir_config, upperdir, workdir
    );

    let mut data = format!("lowerdir={}", lowerdir_config);
    if let (Some(u), Some(w)) = (upperdir, workdir) {
        data = format!("{},upperdir={},workdir={}", data, u.display(), w.display());
    }

    let data_c = CString::new(data).context("Failed to create CString for mount data")?;

    mount(
        KSU_OVERLAY_SOURCE,
        dest,
        "overlay",
        MountFlags::empty(),
        Some(data_c.as_c_str()),
    )
    .with_context(|| format!("Legacy mount failed for {}", dest.display()))?;

    Ok(())
}

pub fn bind_mount(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    debug!(
        "bind mount {} -> {}",
        from.as_ref().display(),
        to.as_ref().display()
    );
    let tree = open_tree(
        CWD,
        from.as_ref(),
        OpenTreeFlags::CLOEXEC | OpenTreeFlags::CLONE | OpenTreeFlags::RECURSIVE,
    )?;
    move_mount(
        tree.as_fd(),
        "",
        CWD,
        to.as_ref(),
        MoveMountFlags::MOVE_MOUNT_F_EMPTY_PATH,
    )?;
    Ok(())
}

fn mount_overlay_on_path(
    mount_point: &Path,
    relative: &str,
    module_roots: &[String],
    stock_root: &Path,
) -> Result<()> {
    if !module_roots
        .iter()
        .any(|lower| Path::new(&format!("{}{}", lower, relative)).exists())
    {
        return Ok(());
    }

    let mut lower_dirs = Vec::new();
    for lower in module_roots {
        let lower_dir = format!("{}{}", lower, relative);
        let path = Path::new(&lower_dir);
        if path.exists() {
            lower_dirs.push(lower_dir);
        }
    }

    if lower_dirs.is_empty() {
        return Ok(());
    }

    if let Err(e) = mount_overlayfs(
        &lower_dirs,
        stock_root.to_str().unwrap(),
        None,
        None,
        mount_point,
    ) {
        warn!(
            "failed to mount overlay on {}: {:#}, fallback to bind",
            mount_point.display(),
            e
        );
        bind_mount(stock_root, mount_point)?;
    }
    Ok(())
}

pub fn mount_overlay(
    target: &str,
    module_roots: &[String],
    _workdir: Option<PathBuf>,
    _upperdir: Option<PathBuf>,
    disable_umount: bool,
) -> Result<()> {
    info!("mount overlay for {} (subdirectory mode)", target);

    let process = Process::myself().context("failed to get self process")?;
    let mount_infos = process.mountinfo().context("failed to get mountinfo")?;
    let mount_points: HashSet<PathBuf> = mount_infos
        .0
        .iter()
        .map(|m| m.mount_point.clone())
        .collect();

    std::env::set_current_dir(target).with_context(|| format!("failed to chdir to {}", target))?;

    let entries = fs::read_dir(".").with_context(|| "failed to read target directory")?;

    let exclusions = ["lost+found"];

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        if exclusions.iter().any(|&e| e == name_str) {
            continue;
        }

        if let Ok(file_type) = entry.file_type() {
            if !file_type.is_dir() {
                continue;
            }
            if file_type.is_symlink() {
                continue;
            }
        } else {
            continue;
        }

        let target_path = Path::new(target).join(&*name_str);

        if mount_points.contains(&target_path) {
            debug!("Skipping mount point: {}", target_path.display());
            continue;
        }

        let relative_path = format!("/{}", name_str);
        let stock_root_path = Path::new(".").join(&*name_str);

        if let Err(e) =
            mount_overlay_on_path(&target_path, &relative_path, module_roots, &stock_root_path)
        {
            warn!("Failed to mount child {}: {}", target_path.display(), e);
        }
    }

    let mount_seq = mount_infos
        .0
        .iter()
        .filter(|m| {
            m.mount_point.starts_with(target) && !Path::new(target).starts_with(&m.mount_point)
        })
        .map(|m| m.mount_point.to_str());

    let mut valid_mount_seq: Vec<&str> = mount_seq.flatten().collect();
    valid_mount_seq.sort();
    valid_mount_seq.dedup();

    for mount_point in valid_mount_seq {
        let relative: String = mount_point.replacen(target, "", 1);
        let path_obj = Path::new(&relative);

        if let Some(first_str) = path_obj
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
        {
            let clean_name = first_str.trim_start_matches('/');
            if exclusions.contains(&clean_name) {
                continue;
            }

            let root_child = Path::new(target).join(clean_name);
            if mount_points.contains(&root_child) {
                continue;
            }
        }

        let child_stock_root = format!(".{}", relative);

        if !Path::new(&child_stock_root).exists() {
            continue;
        }

        if let Err(e) = mount_overlay_on_path(
            Path::new(mount_point),
            &relative,
            module_roots,
            Path::new(&child_stock_root),
        ) {
            warn!(
                "failed to mount overlay for child {}: {:#}, revert",
                mount_point, e
            );
            if !disable_umount {
                let _ = unmount(target, UnmountFlags::empty());
            }
            bail!(e);
        }
    }

    Ok(())
}
