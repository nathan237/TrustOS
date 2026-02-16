#!/bin/bash
set -e

echo "=== Creating TrustOS Linux Subsystem initramfs ==="

# Use script directory (portable, no hardcoded paths)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LINUX_DIR="$SCRIPT_DIR"
INITRAMFS_DIR="$LINUX_DIR/initramfs"

# Clean and create directories
rm -rf "$INITRAMFS_DIR"
mkdir -p "$INITRAMFS_DIR"/{bin,sbin,etc,proc,sys,dev,tmp,usr/bin,usr/sbin,root}

# Copy BusyBox
cp /bin/busybox "$INITRAMFS_DIR/bin/"

# Create symlinks for all BusyBox commands
cd "$INITRAMFS_DIR/bin"
for cmd in $(./busybox --list); do
    ln -sf busybox "$cmd" 2>/dev/null || true
done

# Create init script
cat > "$INITRAMFS_DIR/init" << 'INITEOF'
#!/bin/busybox sh

# Mount essential filesystems
/bin/busybox mkdir -p /proc /sys /dev /tmp /run
/bin/busybox mount -t proc proc /proc
/bin/busybox mount -t sysfs sysfs /sys
/bin/busybox mount -t devtmpfs devtmpfs /dev 2>/dev/null || {
    # Manual device creation if devtmpfs not available
    /bin/busybox mknod -m 666 /dev/null c 1 3
    /bin/busybox mknod -m 666 /dev/zero c 1 5
    /bin/busybox mknod -m 666 /dev/tty c 5 0
    /bin/busybox mknod -m 666 /dev/console c 5 1
    /bin/busybox mknod -m 666 /dev/ttyS0 c 4 64
    /bin/busybox mknod -m 666 /dev/hvc0 c 229 0
}

# Ensure serial device exists
/bin/busybox mknod -m 666 /dev/ttyS0 c 4 64 2>/dev/null || true
/bin/busybox mknod -m 666 /dev/hvc0 c 229 0 2>/dev/null || true

# Setup hostname
/bin/busybox hostname trustos-linux

# Create essential /etc files
echo "root:x:0:0:root:/root:/bin/sh" > /etc/passwd
echo "root:x:0:" > /etc/group
echo "trustos-linux" > /etc/hostname

# Create os-release
cat > /etc/os-release << 'OSREL'
NAME="TrustOS Linux Subsystem"
VERSION="1.0"
ID=trustos-tsl
VERSION_ID="1.0"
PRETTY_NAME="TrustOS Subsystem for Linux (TSL) v1.0"
HOME_URL="https://trustos.dev"
OSREL

# Export environment
export HOME=/root
export PATH=/bin:/sbin:/usr/bin:/usr/sbin
export TERM=linux

# Display welcome message
echo ""
echo "  _____ ____  _     "
echo " |_   _/ ___|| |    "
echo "   | | \\___ \\| |    "
echo "   | |  ___) | |___ "
echo "   |_| |____/|_____|"
echo ""
echo "TrustOS Subsystem for Linux v1.0"
echo "Kernel: $(uname -r 2>/dev/null || echo 'unknown')"
echo "Running inside TrustOS hypervisor"
echo ""
echo "Type 'help' for available commands"
echo ""

# Start interactive shell on serial console
# This allows TrustOS to send commands and receive output
exec /bin/busybox setsid /bin/busybox sh -c 'exec /bin/busybox sh </dev/ttyS0 >/dev/ttyS0 2>&1'
INITEOF

chmod +x "$INITRAMFS_DIR/init"

# Create cpio archive
echo "Creating initramfs archive..."
cd "$INITRAMFS_DIR"
find . | cpio -o -H newc 2>/dev/null | gzip > "$LINUX_DIR/initramfs.cpio.gz"

echo "initramfs created: $(ls -lh $LINUX_DIR/initramfs.cpio.gz | awk '{print $5}')"

# Get a Linux kernel
echo ""
echo "=== Getting Linux kernel ==="
cd "$LINUX_DIR"

# Try different sources for the kernel
if [ -f /boot/vmlinuz-* ]; then
    KERNEL=$(ls /boot/vmlinuz-* 2>/dev/null | sort -V | tail -1)
    echo "Using system kernel: $KERNEL"
    cp "$KERNEL" bzImage
elif [ -f /mnt/c/Windows/System32/lxss/tools/kernel ]; then
    echo "Using WSL kernel..."
    cp /mnt/c/Windows/System32/lxss/tools/kernel bzImage
else
    echo "No local kernel found, downloading..."
    # Try to download a prebuilt minimal kernel
    wget -q --no-check-certificate \
        "https://github.com/nicholaschiasson/linux-kernel/releases/download/v6.1.0/bzImage" \
        -O bzImage 2>/dev/null || {
        echo "Download failed. Please provide a bzImage manually."
        exit 1
    }
fi

if [ -f bzImage ]; then
    echo "Kernel ready: $(ls -lh bzImage | awk '{print $5}')"
fi

# Copy to iso_root
echo ""
echo "=== Copying to iso_root ==="
ISO_LINUX="$(dirname "$LINUX_DIR")/iso_root/boot/linux"
mkdir -p "$ISO_LINUX"
cp "$LINUX_DIR/bzImage" "$ISO_LINUX/" 2>/dev/null || true
cp "$LINUX_DIR/initramfs.cpio.gz" "$ISO_LINUX/" 2>/dev/null || true

echo ""
echo "=== Files created ==="
ls -lh "$LINUX_DIR"/{bzImage,initramfs.cpio.gz} 2>/dev/null || true

echo ""
echo "=== Setup complete! ==="
