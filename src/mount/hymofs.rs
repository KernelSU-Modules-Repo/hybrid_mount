use std::ffi::{CStr, CString};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use log::{debug, warn};
use walkdir::WalkDir;

const DEV_PATH: &str = "/dev/hymo_ctl";
const HYMO_IOC_MAGIC: u8 = 0xE0;
const EXPECTED_PROTOCOL_VERSION: i32 = 4;

const _IOC_NRBITS: u32 = 8;
const _IOC_TYPEBITS: u32 = 8;
const _IOC_SIZEBITS: u32 = 14;
const _IOC_DIRBITS: u32 = 2;

const _IOC_NRSHIFT: u32 = 0;
const _IOC_TYPESHIFT: u32 = _IOC_NRSHIFT + _IOC_NRBITS;
const _IOC_SIZESHIFT: u32 = _IOC_TYPESHIFT + _IOC_TYPEBITS;
const _IOC_DIRSHIFT: u32 = _IOC_SIZESHIFT + _IOC_SIZEBITS;

const _IOC_NONE: u32 = 0;
const _IOC_WRITE: u32 = 1;
const _IOC_READ: u32 = 2;
const _IOC_READ_WRITE: u32 = 3;

macro_rules! _IOC {
    ($dir:expr, $type:expr, $nr:expr, $size:expr) => {
        (($dir) << _IOC_DIRSHIFT) |
        (($type) << _IOC_TYPESHIFT) |
        (($nr) << _IOC_NRSHIFT) |
        (($size) << _IOC_SIZESHIFT)
    };
}

macro_rules! _IO {
    ($type:expr, $nr:expr) => {
        _IOC!(_IOC_NONE, $type, $nr, 0)
    };
}

macro_rules! _IOR {
    ($type:expr, $nr:expr, $size:ty) => {
        _IOC!(_IOC_READ, $type, $nr, std::mem::size_of::<$size>() as u32)
    };
}

macro_rules! _IOW {
    ($type:expr, $nr:expr, $size:ty) => {
        _IOC!(_IOC_WRITE, $type, $nr, std::mem::size_of::<$size>() as u32)
    };
}

macro_rules! _IOWR {
    ($type:expr, $nr:expr, $size:ty) => {
        _IOC!(_IOC_READ_WRITE, $type, $nr, std::mem::size_of::<$size>() as u32)
    };
}

#[repr(C)]
struct HymoIoctlArg {
    src: *const libc::c_char,
    target: *const libc::c_char,
    r#type: u8,
}

#[repr(C)]
struct HymoIoctlListArg {
    buf: *mut libc::c_char,
    size: usize,
}

fn ioc_add_rule() -> libc::c_int { _IOW!(HYMO_IOC_MAGIC as u32, 1, HymoIoctlArg) as libc::c_int }
fn ioc_del_rule() -> libc::c_int { _IOW!(HYMO_IOC_MAGIC as u32, 2, HymoIoctlArg) as libc::c_int }
fn ioc_hide_rule() -> libc::c_int { _IOW!(HYMO_IOC_MAGIC as u32, 3, HymoIoctlArg) as libc::c_int }
fn ioc_inject_rule() -> libc::c_int { _IOW!(HYMO_IOC_MAGIC as u32, 4, HymoIoctlArg) as libc::c_int }
fn ioc_clear_all() -> libc::c_int { _IO!(HYMO_IOC_MAGIC as u32, 5) as libc::c_int }
fn ioc_get_version() -> libc::c_int { _IOR!(HYMO_IOC_MAGIC as u32, 6, libc::c_int) as libc::c_int }
fn ioc_list_rules() -> libc::c_int { _IOWR!(HYMO_IOC_MAGIC as u32, 7, HymoIoctlListArg) as libc::c_int }

#[derive(Debug)]
pub enum HymoRule {
    Redirect {
        src: PathBuf,
        target: PathBuf,
        file_type: i32,
    },
    Hide {
        path: PathBuf,
    },
    Inject {
        dir: PathBuf,
    },
    Delete {
        src: PathBuf,
    }
}

struct HymoDriver {
    file: File,
}

impl HymoDriver {
    fn new() -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(DEV_PATH)
            .with_context(|| format!("Failed to open HymoFS control device: {}", DEV_PATH))?;
        Ok(Self { file })
    }

    fn get_version(&self) -> Result<i32> {
        let mut version: libc::c_int = 0;
        let ret = unsafe {
            libc::ioctl(self.file.as_raw_fd(), ioc_get_version(), &mut version)
        };
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            anyhow::bail!("Failed to get version via ioctl: {}", err);
        }
        Ok(version)
    }

    fn clear(&self) -> Result<()> {
        let ret = unsafe {
            libc::ioctl(self.file.as_raw_fd(), ioc_clear_all())
        };
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            anyhow::bail!("Failed to clear rules: {}", err);
        }
        Ok(())
    }

    fn apply_rules(&self, rules: &[HymoRule]) -> Result<()> {
        for rule in rules {
            match rule {
                HymoRule::Redirect { src, target, file_type } => {
                    let c_src = CString::new(src.to_string_lossy().as_bytes())?;
                    let c_target = CString::new(target.to_string_lossy().as_bytes())?;
                    let arg = HymoIoctlArg {
                        src: c_src.as_ptr(),
                        target: c_target.as_ptr(),
                        r#type: *file_type as u8,
                    };
                    let ret = unsafe { libc::ioctl(self.file.as_raw_fd(), ioc_add_rule(), &arg) };
                    if ret < 0 {
                        log::warn!("HymoFS Add failed for {:?}: {}", src, std::io::Error::last_os_error());
                    }
                }
                HymoRule::Hide { path } => {
                    let c_path = CString::new(path.to_string_lossy().as_bytes())?;
                    let arg = HymoIoctlArg {
                        src: c_path.as_ptr(),
                        target: std::ptr::null(),
                        r#type: 0,
                    };
                    let ret = unsafe { libc::ioctl(self.file.as_raw_fd(), ioc_hide_rule(), &arg) };
                    if ret < 0 {
                        log::warn!("HymoFS Hide failed for {:?}: {}", path, std::io::Error::last_os_error());
                    }
                }
                HymoRule::Inject { dir } => {
                    let c_dir = CString::new(dir.to_string_lossy().as_bytes())?;
                    let arg = HymoIoctlArg {
                        src: c_dir.as_ptr(),
                        target: std::ptr::null(),
                        r#type: 0,
                    };
                    let ret = unsafe { libc::ioctl(self.file.as_raw_fd(), ioc_inject_rule(), &arg) };
                    if ret < 0 {
                        log::warn!("HymoFS Inject failed for {:?}: {}", dir, std::io::Error::last_os_error());
                    }
                }
                HymoRule::Delete { src } => {
                    let c_src = CString::new(src.to_string_lossy().as_bytes())?;
                    let arg = HymoIoctlArg {
                        src: c_src.as_ptr(),
                        target: std::ptr::null(),
                        r#type: 0,
                    };
                    let ret = unsafe { libc::ioctl(self.file.as_raw_fd(), ioc_del_rule(), &arg) };
                    if ret < 0 {
                        log::warn!("HymoFS Delete failed for {:?}: {}", src, std::io::Error::last_os_error());
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn list_rules(&self) -> Result<String> {
        let capacity = 32 * 1024; // 32KB buffer
        let mut buffer = vec![0u8; capacity];
        let mut arg = HymoIoctlListArg {
            buf: buffer.as_mut_ptr() as *mut libc::c_char,
            size: capacity,
        };

        let ret = unsafe {
            libc::ioctl(self.file.as_raw_fd(), ioc_list_rules(), &mut arg)
        };

        if ret < 0 {
            let err = std::io::Error::last_os_error();
            anyhow::bail!("Failed to list rules via ioctl: {}", err);
        }

        let c_str = unsafe { CStr::from_ptr(buffer.as_ptr() as *const libc::c_char) };
        Ok(c_str.to_string_lossy().into_owned())
    }
}

#[derive(Debug, PartialEq)]
pub enum HymoFsStatus {
    Available,
    NotPresent,
    KernelTooOld,
    ModuleTooOld,
}

pub struct HymoFs;

impl HymoFs {
    #[allow(dead_code)]
    pub fn get_version() -> Option<i32> {
        let driver = HymoDriver::new().ok()?;
        driver.get_version().ok()
    }

    pub fn check_status() -> HymoFsStatus {
        let driver = match HymoDriver::new() {
            Ok(d) => d,
            Err(_) => return HymoFsStatus::NotPresent,
        };

        let version = match driver.get_version() {
            Ok(v) => v,
            Err(_) => return HymoFsStatus::NotPresent,
        };

        if version != EXPECTED_PROTOCOL_VERSION {
            warn!(
                "HymoFS protocol mismatch! Kernel: {}, User: {}",
                version, EXPECTED_PROTOCOL_VERSION
            );
            if version < EXPECTED_PROTOCOL_VERSION {
                return HymoFsStatus::KernelTooOld;
            } else {
                return HymoFsStatus::ModuleTooOld;
            }
        }

        HymoFsStatus::Available
    }

    pub fn is_available() -> bool {
        Self::check_status() == HymoFsStatus::Available
    }

    pub fn clear() -> Result<()> {
        let driver = HymoDriver::new()?;
        driver.clear()
    }

    pub fn inject_directory(target_base: &Path, module_dir: &Path) -> Result<()> {
        if !module_dir.exists() || !module_dir.is_dir() {
            return Ok(());
        }

        let driver = HymoDriver::new()?;
        let mut rules = Vec::new();

        rules.push(HymoRule::Inject {
            dir: target_base.to_path_buf(),
        });

        for entry in WalkDir::new(module_dir).min_depth(1) {
            let entry = entry?;
            let relative_path = entry.path().strip_prefix(module_dir)?;
            let target_path = target_base.join(relative_path);
            let file_type = entry.file_type();

            if file_type.is_char_device() {
                let metadata = entry.metadata()?;
                if metadata.rdev() == 0 {
                    rules.push(HymoRule::Hide {
                        path: target_path,
                    });
                }
            } else if file_type.is_dir() {
                rules.push(HymoRule::Inject {
                    dir: target_path.clone(),
                });
                
                rules.push(HymoRule::Redirect {
                    src: target_path,
                    target: entry.path().to_path_buf(),
                    file_type: 4, // DT_DIR
                });
            } else {
                let type_code = if file_type.is_symlink() {
                    10 // DT_LNK
                } else {
                    8  // DT_REG
                };

                rules.push(HymoRule::Redirect {
                    src: target_path,
                    target: entry.path().to_path_buf(),
                    file_type: type_code,
                });
            }
        }

        driver.apply_rules(&rules).context("Failed to apply HymoFS rules")?;
        
        debug!("Injected {} rules for {}", rules.len(), target_base.display());
        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete_directory_rules(target_base: &Path, module_dir: &Path) -> Result<()> {
        if !module_dir.exists() || !module_dir.is_dir() {
            return Ok(());
        }

        let driver = HymoDriver::new()?;
        let mut rules = Vec::new();

        for entry in WalkDir::new(module_dir).min_depth(1) {
            let entry = entry?;
            let relative_path = entry.path().strip_prefix(module_dir)?;
            let target_path = target_base.join(relative_path);
            let file_type = entry.file_type();

            if file_type.is_file() || file_type.is_symlink() {
                rules.push(HymoRule::Delete { src: target_path });
            } else if file_type.is_char_device() {
                let metadata = entry.metadata()?;
                if metadata.rdev() == 0 {
                    rules.push(HymoRule::Delete { src: target_path });
                }
            }
        }

        driver.apply_rules(&rules).context("Failed to delete HymoFS rules")?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn hide_path(path: &Path) -> Result<()> {
        let driver = HymoDriver::new()?;
        let rules = vec![HymoRule::Hide {
            path: path.to_path_buf(),
        }];
        driver.apply_rules(&rules)
    }

    #[allow(dead_code)]
    pub fn list_active_rules() -> Result<String> {
        let driver = HymoDriver::new()?;
        driver.list_rules()
    }
}
