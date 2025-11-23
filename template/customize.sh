#!/system/bin/sh

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
