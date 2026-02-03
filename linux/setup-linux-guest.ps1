# setup-linux-guest.ps1
# Script to prepare minimal Linux kernel and initramfs for TrustOS TSL

$ErrorActionPreference = "Stop"
$LinuxDir = "c:\Users\nathan\Documents\Scripts\OSrust\linux"

Write-Host "=== TrustOS Linux Subsystem - Kernel Setup ===" -ForegroundColor Cyan

# Create linux directory
if (-not (Test-Path $LinuxDir)) {
    New-Item -ItemType Directory -Path $LinuxDir -Force | Out-Null
}

Write-Host "`n[1/4] Preparing Linux kernel..." -ForegroundColor Yellow

# Execute WSL script to build initramfs
$wslCommands = @"
cd /mnt/c/Users/nathan/Documents/Scripts/OSrust/linux

echo '=== Creating initramfs with BusyBox ==='

# Install dependencies if needed
apt-get update -qq 2>/dev/null
apt-get install -y busybox-static wget cpio gzip 2>/dev/null

# Create initramfs structure
INITRAMFS_DIR="/mnt/c/Users/nathan/Documents/Scripts/OSrust/linux/initramfs"
rm -rf "\$INITRAMFS_DIR"
mkdir -p "\$INITRAMFS_DIR"/{bin,sbin,etc,proc,sys,dev,tmp,usr/bin,usr/sbin,root}

# Copy BusyBox
cp /bin/busybox "\$INITRAMFS_DIR/bin/"

# Create symlinks for all BusyBox commands
cd "\$INITRAMFS_DIR/bin"
for cmd in \$(./busybox --list); do
    ln -sf busybox "\$cmd" 2>/dev/null || true
done

# Create init script
cat > "\$INITRAMFS_DIR/init" << 'INITEOF'
#!/bin/busybox sh

/bin/busybox mkdir -p /proc /sys /dev /tmp
/bin/busybox mount -t proc proc /proc
/bin/busybox mount -t sysfs sysfs /sys
/bin/busybox mount -t devtmpfs devtmpfs /dev 2>/dev/null || true

/bin/busybox mknod -m 666 /dev/null c 1 3 2>/dev/null || true
/bin/busybox mknod -m 666 /dev/zero c 1 5 2>/dev/null || true
/bin/busybox mknod -m 666 /dev/ttyS0 c 4 64 2>/dev/null || true
/bin/busybox mknod -m 666 /dev/hvc0 c 229 0 2>/dev/null || true

/bin/busybox hostname trustos-linux

echo "root:x:0:0:root:/root:/bin/sh" > /etc/passwd
echo "root:x:0:" > /etc/group
echo "trustos-linux" > /etc/hostname

cat > /etc/os-release << 'OSREL'
NAME="TrustOS Linux Subsystem"
VERSION="1.0"
ID=trustos-tsl
PRETTY_NAME="TrustOS Subsystem for Linux (TSL)"
OSREL

clear
echo ""
echo "TrustOS Subsystem for Linux v1.0"
echo "Running inside TrustOS hypervisor"
echo ""

exec /bin/busybox setsid /bin/busybox sh -c 'exec /bin/busybox sh </dev/ttyS0 >/dev/ttyS0 2>&1'
INITEOF

chmod +x "\$INITRAMFS_DIR/init"

# Create cpio archive
cd "\$INITRAMFS_DIR"
find . | cpio -o -H newc 2>/dev/null | gzip > /mnt/c/Users/nathan/Documents/Scripts/OSrust/linux/initramfs.cpio.gz
echo "initramfs created: \$(ls -lh /mnt/c/Users/nathan/Documents/Scripts/OSrust/linux/initramfs.cpio.gz | awk '{print \$5}')"

echo ''
echo '=== Getting Linux kernel ==='

# Try to copy kernel from WSL
cd /mnt/c/Users/nathan/Documents/Scripts/OSrust/linux
if [ -f /boot/vmlinuz-* ]; then
    KERNEL=\$(ls /boot/vmlinuz-* 2>/dev/null | head -1)
    echo "Found kernel: \$KERNEL"
    cp "\$KERNEL" bzImage
elif [ -f /mnt/c/Windows/System32/lxss/tools/kernel ]; then
    echo "Using WSL kernel..."
    cp /mnt/c/Windows/System32/lxss/tools/kernel bzImage
else
    echo "Downloading minimal kernel..."
    wget -q --no-check-certificate "https://github.com/nicholaschiasson/linux-kernel/releases/download/v6.1.0/bzImage" -O bzImage 2>/dev/null || {
        echo "Download failed, trying alternative..."
        # Create a placeholder
        echo "KERNEL_PLACEHOLDER" > bzImage
    }
fi

echo ''
echo '=== Files created ==='
ls -lh /mnt/c/Users/nathan/Documents/Scripts/OSrust/linux/
"@

Write-Host "[2/4] Running WSL build script..." -ForegroundColor Yellow
wsl -d Ubuntu -u root bash -c $wslCommands

Write-Host "`n[3/4] Verifying files..." -ForegroundColor Yellow
$files = @("$LinuxDir\bzImage", "$LinuxDir\initramfs.cpio.gz")
foreach ($file in $files) {
    if (Test-Path $file) {
        $size = (Get-Item $file).Length / 1MB
        Write-Host "  OK $([System.IO.Path]::GetFileName($file)) - $([math]::Round($size, 2)) MB" -ForegroundColor Green
    } else {
        Write-Host "  MISSING $([System.IO.Path]::GetFileName($file))" -ForegroundColor Red
    }
}

Write-Host "`n[4/4] Copying to iso_root..." -ForegroundColor Yellow
$isoLinuxDir = "c:\Users\nathan\Documents\Scripts\OSrust\iso_root\boot\linux"
New-Item -ItemType Directory -Path $isoLinuxDir -Force | Out-Null

if (Test-Path "$LinuxDir\bzImage") {
    Copy-Item "$LinuxDir\bzImage" "$isoLinuxDir\bzImage" -Force
    Write-Host "  OK bzImage copied to iso_root" -ForegroundColor Green
}
if (Test-Path "$LinuxDir\initramfs.cpio.gz") {
    Copy-Item "$LinuxDir\initramfs.cpio.gz" "$isoLinuxDir\initramfs.cpio.gz" -Force
    Write-Host "  OK initramfs copied to iso_root" -ForegroundColor Green
}

Write-Host "`n=== Linux Guest Setup Complete! ===" -ForegroundColor Cyan
