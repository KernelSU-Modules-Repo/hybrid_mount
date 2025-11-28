// meta-hybrid_mount/src/defs.rs

// Hybrid Mount Constants

// NOTE: The actual content directory is now determined dynamically at runtime 
// (e.g. searching for decoy paths). This is a fallback default.
pub const FALLBACK_CONTENT_DIR: &str = "/data/adb/meta-hybrid/mnt/";

// The base directory for our own config and logs
pub const BASE_DIR: &str = "/data/adb/meta-hybrid/";

// Log file path (Must match WebUI)
pub const DAEMON_LOG_FILE: &str = "/data/adb/meta-hybrid/daemon.log";

// Markers
pub const DISABLE_FILE_NAME: &str = "disable";
pub const REMOVE_FILE_NAME: &str = "remove";
pub const SKIP_MOUNT_FILE_NAME: &str = "skip_mount";

// OverlayFS Source Name
pub const OVERLAY_SOURCE: &str = "KSU";

// --- Fixes for compilation errors ---
pub const KSU_OVERLAY_SOURCE: &str = OVERLAY_SOURCE;

// Path for overlayfs workdir/upperdir (if needed in future)
#[allow(dead_code)]
pub const SYSTEM_RW_DIR: &str = "/data/adb/meta-hybrid/rw";

// LKM Paths
// This points to where the kernel modules are installed in the Magisk/KSU module directory.
pub const MODULE_LKM_DIR: &str = "/data/adb/modules/meta-hybrid/lkm/binaries";