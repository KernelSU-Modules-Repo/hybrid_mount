#!/system/bin/sh
############################################
# meta-hybrid metainstall.sh
############################################

# This tells KernelSU it's a metamodule
export KSU_HAS_METAMODULE="true"
export KSU_METAMODULE="meta-hybrid"

# Constants
BASE_DIR="/data/adb/meta-hybrid"

# 1. The installation process
ui_print "- Using Hybrid Mount metainstall"

# Standard installation (extracts to /data/adb/modules/meta-hybrid)
install_module

# NOTE: We no longer move files to modules.img at install time.
# The Rust daemon will handle copying files to Tmpfs or Image at boot time.
# This ensures we always have the source files available for fallback logic.

ui_print "- Installation complete"