#!/system/bin/sh

# === Volume Key Detection Function ===
chooseport() {
  # Keycheck binary from standard Magisk module template logic
  # We use the existing keycheck if available or try a simple timeout method if not perfectly reliable
  # But for simplicity in customize.sh, we rely on the standard "getevent" method available in Android
  
  # Original idea from various Magisk module templates
  # $1 = delay in seconds
  
  local delay=$1
  local start_time=$(date +%s)
  local current_time
  
  ui_print "- Press Vol UP to CONFIRM, or wait 10s to ABORT."
  
  while true; do
    current_time=$(date +%s)
    if [ $((current_time - start_time)) -ge $delay ]; then
      return 1 # Timeout
    fi
    
    # Check for Volume Up key press
    # This is a simplified check. For robust keycheck, modules often ship a binary.
    # Here we assume a standard environment or simple timeout is the main constraint.
    # Since we can't easily ship a binary in this snippet, we will try to use `getevent`.
    
    # Note: getevent behavior varies by device. 
    # If getevent -lc 1 returns an event containing KEY_VOLUMEUP, we succeed.
    # However, getevent is blocking. We use 'timeout' if available or a loop.
    
    # A more robust "pure shell" way without a binary is hard because getevent blocks.
    # Most modules carry a 'keycheck' binary. 
    # If you don't have one, the below logic might be flaky on some devices.
    # I will implement a "best effort" using timeout + getevent if available.
    
    if timeout 0.1 getevent -lc 1 2>/dev/null | grep -q "KEY_VOLUMEUP"; then
      return 0
    fi
    
    # Alternative: check for KEY_VOLUMEDOWN to abort immediately? 
    # Requirement only said Vol UP to confirm.
  done
}

# Better implementation using standard "keycheck" logic found in most installers
# Since I cannot see if you have a 'keycheck' binary in your zip, I will use a
# widespread shell-only approximation, but please note:
# RELIABLE VOLUME KEY DETECTION USUALLY REQUIRES A COMPILED BINARY (keycheck).
# If this script hangs, you need that binary.
# 
# simplified version for the request:

ui_print "*********************************************************"
ui_print "* 免责声明 / DISCLAIMER                *"
ui_print "*********************************************************"
ui_print "* 1. 本模块仅供学习交流，造成任何损坏后果自负。"
ui_print "* 2. 禁止在【酷安 (CoolApk)】平台传播此模块！"
ui_print "* 若发现一次违规传播，将停更一个月。"
ui_print "*********************************************************"
ui_print ""
ui_print "- 请在 10 秒内按【音量+】键同意并安装..."
ui_print "- Press Vol+ within 10s to agree and install..."

# Set up keycheck
KEYCHECK_PASSED=0
# We use a loop with a total timeout
count=0
limit=100 # 10 seconds * 10 checks/sec (approx)

while [ $count -lt $limit ]; do
    # Try to grab one event. 'timeout' command is usually available in recovery/magisk environment
    # getevent -l -c 1 prints 1 event. 
    # We filter for VOLUMEUP.
    # Note: different devices output different codes (0073, KEY_VOLUMEUP, etc)
    # This grep catches the string "VOLUMEUP" or hex code 0073 (standard linux)
    
    # Using 'timeout' to prevent getevent from blocking forever if no key pressed
    event=$(timeout 0.1 getevent -l -c 1 2>&1)
    
    if echo "$event" | grep -q -E "VOLUMEUP|0073"; then
        KEYCHECK_PASSED=1
        break
    fi
    
    count=$((count + 1))
done

if [ $KEYCHECK_PASSED -eq 1 ]; then
    ui_print "- 已确认免责声明，开始安装..."
else
    ui_print "! 超时未确认或按键识别失败。"
    ui_print "! 安装已取消。"
    abort "! Installation aborted by user or timeout."
fi

# === Original Logic Below ===

ui_print "- Detecting device architecture..."

# Detect architecture using ro.product.cpu.abi
ABI=$(grep_get_prop ro.product.cpu.abi)
ui_print "- Detected ABI: $ABI"

ui_print "- Installing $ARCH_BINARY as meta-mm"

# Rename the selected binary to the generic name
mv "$MODPATH/magic_mount_rs" "$MODPATH/meta-mm" || abort "! Failed to rename binary"

# Ensure the binary is executable
chmod 755 "$MODPATH/meta-mm" || abort "! Failed to set permissions"

ui_print "- Architecture-specific binary installed successfully"

mkdir -p /data/adb/magic_mount

if [ ! -f /data/adb/magic_mount/config.toml ] ; then
  ui_print "- Add default config"
  cat "$MODPATH/config.toml" > /data/adb/magic_mount/config.toml
fi

ui_print "- Installation complete"
ui_print "- Image is ready for module installations"
