use std::{
    fs::{create_dir_all, remove_dir_all, remove_file, write, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

use anyhow::{Context, Result, bail};
use rustix::mount::{mount, MountFlags, unmount, UnmountFlags};
#[cfg(any(target_os = "linux", target_os = "android"))]
use extattr::{Flags as XattrFlags, lsetxattr};

const SELINUX_XATTR: &str = "security.selinux";
// Test file to check xattr support
const XATTR_TEST_FILE: &str = ".xattr_test";

// --- File Logger Implementation ---
struct FileLogger {
    file: Mutex<std::fs::File>,
}

impl log::Log for FileLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut file = self.file.lock().unwrap();
            let _ = writeln!(
                file,
                "[{}] [{}] {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {
        let _ = self.file.lock().unwrap().flush();
    }
}

pub fn init_logger(verbose: bool, log_path: &Path) -> Result<()> {
    let level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    if let Some(parent) = log_path.parent() {
        create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true) // Append mode
        .open(log_path)
        .with_context(|| format!("Failed to open log file: {}", log_path.display()))?;

    let logger = Box::new(FileLogger {
        file: Mutex::new(file),
    });

    log::set_boxed_logger(logger)
        .map(|()| log::set_max_level(level))
        .map_err(|e| anyhow::anyhow!("Failed to set logger: {}", e))?;

    Ok(())
}

pub fn lsetfilecon<P: AsRef<Path>>(path: P, con: &str) -> Result<()> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        if let Err(e) = lsetxattr(&path, SELINUX_XATTR, con, XattrFlags::empty()) {
            let io_err = std::io::Error::from(e);
            if io_err.kind() == std::io::ErrorKind::PermissionDenied {
                log::warn!("SELinux permission denied for {} (ignored)", path.as_ref().display());
            } else {
                // Don't error out, just warn. Critical for tmpfs fallback logic if xattr fails unexpectedly.
                log::warn!("Failed to set SELinux context for {}: {}", path.as_ref().display(), io_err);
            }
        }
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn lgetfilecon<P: AsRef<Path>>(path: P) -> Result<String> {
    let con = extattr::lgetxattr(&path, SELINUX_XATTR).with_context(|| {
        format!("Failed to get SELinux context for {}", path.as_ref().display())
    })?;
    Ok(String::from_utf8_lossy(&con).to_string())
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub fn lgetfilecon<P: AsRef<Path>>(_path: P) -> Result<String> {
    unimplemented!()
}

pub fn ensure_dir_exists<T: AsRef<Path>>(dir: T) -> Result<()> {
    if !dir.as_ref().exists() {
        create_dir_all(&dir)?;
    }
    Ok(())
}

// --- Storage Utils ---

/// Checks if the filesystem at `path` supports SELinux extended attributes.
/// Creates a dummy file, tries to set xattr, and cleans up.
pub fn is_xattr_supported(path: &Path) -> bool {
    let test_file = path.join(XATTR_TEST_FILE);
    if let Err(_) = write(&test_file, b"test") {
        return false;
    }

    let supported = lsetfilecon(&test_file, "u:object_r:system_file:s0").is_ok();
    let _ = remove_file(test_file);
    supported
}

/// Mounts Tmpfs at the target directory.
pub fn mount_tmpfs(target: &Path) -> Result<()> {
    ensure_dir_exists(target)?;
    // Size is usually dynamic, but we can set a generous limit if needed.
    // Defaulting to shared memory limits.
    mount("tmpfs", target, "tmpfs", MountFlags::empty(), "mode=0755")
        .context("Failed to mount tmpfs")?;
    Ok(())
}

/// Mounts an ext4 loop image at the target directory.
pub fn mount_image(image_path: &Path, target: &Path) -> Result<()> {
    ensure_dir_exists(target)?;
    
    // We use the shell `mount` command for simplicity with loop devices,
    // as rustix mount requires setting up the loop device manually via ioctl.
    let status = Command::new("mount")
        .args(["-t", "ext4", "-o", "loop,rw,noatime"])
        .arg(image_path)
        .arg(target)
        .status()
        .context("Failed to execute mount command")?;

    if !status.success() {
        bail!("Mount command failed");
    }
    Ok(())
}

/// Synchronizes a directory from source to dest using `cp -af`.
/// Used to copy module files from /data/adb/modules to Tmpfs/Image.
pub fn sync_dir(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() { return Ok(()); }
    ensure_dir_exists(dst)?;

    // Use cp -af to preserve all attributes (recursive, no dereference, preserve all)
    // -a: archive mode (recursive, preserves symlinks, perms, times, context)
    // -f: force
    // Note: 'cp' in Android usually supports -a (toybox/busybox)
    let status = Command::new("cp")
        .arg("-af")
        .arg(format!("{}/.", src.display())) // Copy content, not the dir itself
        .arg(dst)
        .status()
        .context("Failed to execute cp command")?;

    if !status.success() {
        bail!("Failed to sync {} to {}", src.display(), dst.display());
    }
    Ok(())
}

pub fn cleanup_temp_dir(temp_dir: &Path) {
    if let Err(e) = remove_dir_all(temp_dir) {
        log::warn!("Failed to clean up temp dir {}: {:#}", temp_dir.display(), e);
    }
}

pub fn ensure_temp_dir(temp_dir: &Path) -> Result<()> {
    if temp_dir.exists() {
        remove_dir_all(temp_dir).ok();
    }
    create_dir_all(temp_dir)?;
    Ok(())
}

pub fn select_temp_dir() -> Result<PathBuf> {
    // Basic implementation for now, usually /debug_ramdisk or similar
    // Since we are moving to Smart Storage, the logic in main.rs handles the big mounts.
    // This is for magic mount working dirs.
    Ok(PathBuf::from("/debug_ramdisk/meta_hybrid_work"))
}
