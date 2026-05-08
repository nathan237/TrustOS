# TrustOS - Flash ISO to USB Drive
# Usage: .\flash-usb.ps1 [-DiskNumber <N>] [-IsoPath <path>]
# WARNING: This will ERASE the target USB drive completely!

param(
    [int]$DiskNumber = -1,
    [string]$IsoPath = "trustos.iso"
)

$ErrorActionPreference = "Stop"

Write-Host "`n=== TRUSTOS USB FLASHER ===" -ForegroundColor Cyan
Write-Host "This will create a bootable USB drive with TrustOS.`n" -ForegroundColor Yellow

# Check ISO exists
if (-not (Test-Path $IsoPath)) {
    Write-Host "ISO not found: $IsoPath" -ForegroundColor Red
    Write-Host "Run '.\build-limine.ps1 -NoRun' first to build the ISO." -ForegroundColor Yellow
    exit 1
}

$isoSize = (Get-Item $IsoPath).Length
Write-Host "ISO: $IsoPath ($([math]::Round($isoSize / 1MB, 1)) MB)" -ForegroundColor Green

# List removable USB disks
Write-Host "`n--- Removable USB Drives ---" -ForegroundColor Cyan
$usbDisks = Get-Disk | Where-Object { $_.BusType -eq 'USB' -and $_.Size -gt 0 }

if (-not $usbDisks -or $usbDisks.Count -eq 0) {
    Write-Host "No USB drives detected! Insert a USB drive and try again." -ForegroundColor Red
    exit 1
}

$usbDisks | ForEach-Object {
    $sizeGB = [math]::Round($_.Size / 1GB, 1)
    $status = if ($_.IsOffline) { "OFFLINE" } else { "ONLINE" }
    Write-Host ("  Disk {0}: {1} ({2} GB) [{3}] - {4}" -f $_.Number, $_.FriendlyName, $sizeGB, $status, $_.PartitionStyle) -ForegroundColor White
}

# Select disk
if ($DiskNumber -eq -1) {
    Write-Host ""
    $DiskNumber = Read-Host "Enter the disk number of the USB drive to flash"
    $DiskNumber = [int]$DiskNumber
}

# Verify it's a USB disk
$targetDisk = Get-Disk -Number $DiskNumber -ErrorAction SilentlyContinue
if (-not $targetDisk) {
    Write-Host "Disk $DiskNumber not found!" -ForegroundColor Red
    exit 1
}

if ($targetDisk.BusType -ne 'USB') {
    Write-Host "WARNING: Disk $DiskNumber is NOT a USB drive (BusType: $($targetDisk.BusType))!" -ForegroundColor Red
    Write-Host "Refusing to flash non-USB disk for safety." -ForegroundColor Red
    exit 1
}

$sizeGB = [math]::Round($targetDisk.Size / 1GB, 1)
Write-Host "`nTarget: Disk $DiskNumber - $($targetDisk.FriendlyName) ($sizeGB GB)" -ForegroundColor Yellow
Write-Host "ALL DATA ON THIS DRIVE WILL BE DESTROYED!" -ForegroundColor Red

$confirm = Read-Host "`nType 'YES' to confirm flash"
if ($confirm -ne 'YES') {
    Write-Host "Aborted." -ForegroundColor Yellow
    exit 0
}

# ======== Flash ISO to USB (dd-style) ========
Write-Host "`n[1/3] Cleaning disk..." -ForegroundColor Yellow

# Use diskpart to clean AND lock the volume for raw write
$dpScript = @"
select disk $DiskNumber
clean
select disk $DiskNumber
create partition primary
select partition 1
"@
$dpScript | diskpart | Out-Null
Start-Sleep -Seconds 2

# Now remove the partition so it's a raw disk, but Windows has "touched" it
$dpScript2 = @"
select disk $DiskNumber
clean
"@
$dpScript2 | diskpart | Out-Null
Start-Sleep -Seconds 2

Write-Host "[2/3] Writing ISO to USB..." -ForegroundColor Yellow

# Try WSL dd first (most reliable), fallback to .NET raw write
$isoFullPath = (Resolve-Path $IsoPath).Path

function Convert-ToWslPath([string]$winPath) {
    $drive = $winPath.Substring(0, 1).ToLower()
    $rest = $winPath.Substring(2) -replace '\\', '/'
    return "/mnt/$drive$rest"
}

$wslAvailable = Get-Command wsl -ErrorAction SilentlyContinue
$writeSuccess = $false

if ($wslAvailable) {
    Write-Host "Using WSL dd..." -ForegroundColor Cyan
    $wslIso = Convert-ToWslPath $isoFullPath
    $wslDisk = "/dev/sd" + [char]([int][char]'a' + $DiskNumber)
    
    # WSL may not see Windows physical disks as /dev/sdX — use /dev/sgN or pass-through
    # Most reliable: use wsl dd with Windows path via /mnt/
    $isoSizeMB = [math]::Round((Get-Item $isoFullPath).Length / 1MB, 1)
    Write-Host "Writing $isoSizeMB MB to PhysicalDrive$DiskNumber..." -ForegroundColor Yellow
    
    # Use PowerShell .NET with DeviceIoControl FSCTL_LOCK_VOLUME approach
    if (-not ([System.Management.Automation.PSTypeName]'RawDiskWriter').Type) {
        Add-Type -TypeDefinition @"
using System;
using System.IO;
using System.Runtime.InteropServices;

public class RawDiskWriter {
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    static extern IntPtr CreateFile(string lpFileName, uint dwDesiredAccess,
        uint dwShareMode, IntPtr lpSecurityAttributes, uint dwCreationDisposition,
        uint dwFlagsAndAttributes, IntPtr hTemplateFile);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool WriteFile(IntPtr hFile, byte[] lpBuffer,
        uint nNumberOfBytesToWrite, out uint lpNumberOfBytesWritten, IntPtr lpOverlapped);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool DeviceIoControl(IntPtr hDevice, uint dwIoControlCode,
        IntPtr lpInBuffer, uint nInBufferSize, IntPtr lpOutBuffer,
        uint nOutBufferSize, out uint lpBytesReturned, IntPtr lpOverlapped);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool CloseHandle(IntPtr hObject);

    const uint GENERIC_READ = 0x80000000;
    const uint GENERIC_WRITE = 0x40000000;
    const uint FILE_SHARE_READ = 0x00000001;
    const uint FILE_SHARE_WRITE = 0x00000002;
    const uint OPEN_EXISTING = 3;
    const uint FILE_FLAG_WRITE_THROUGH = 0x80000000;
    const uint FILE_FLAG_NO_BUFFERING = 0x20000000;
    const uint FSCTL_LOCK_VOLUME = 0x00090018;
    const uint FSCTL_DISMOUNT_VOLUME = 0x00090020;

    public static long WriteImage(string diskPath, string isoPath) {
        IntPtr hDisk = CreateFile(diskPath,
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            IntPtr.Zero, OPEN_EXISTING,
            FILE_FLAG_WRITE_THROUGH | FILE_FLAG_NO_BUFFERING,
            IntPtr.Zero);

        if (hDisk == new IntPtr(-1)) {
            int err = Marshal.GetLastWin32Error();
            throw new Exception("Failed to open disk: Win32 error " + err);
        }

        try {
            // Lock and dismount volume to ensure exclusive access
            uint dummy;
            DeviceIoControl(hDisk, FSCTL_LOCK_VOLUME, IntPtr.Zero, 0, IntPtr.Zero, 0, out dummy, IntPtr.Zero);
            DeviceIoControl(hDisk, FSCTL_DISMOUNT_VOLUME, IntPtr.Zero, 0, IntPtr.Zero, 0, out dummy, IntPtr.Zero);

            using (var fs = File.OpenRead(isoPath)) {
                byte[] buffer = new byte[1048576]; // 1MB
                long totalWritten = 0;
                int bytesRead;
                while ((bytesRead = fs.Read(buffer, 0, buffer.Length)) > 0) {
                    int aligned = ((bytesRead + 511) / 512) * 512;
                    if (aligned > bytesRead)
                        Array.Clear(buffer, bytesRead, aligned - bytesRead);
                    uint written;
                    if (!WriteFile(hDisk, buffer, (uint)aligned, out written, IntPtr.Zero)) {
                        throw new Exception("Write failed at offset " + totalWritten + ": error " + Marshal.GetLastWin32Error());
                    }
                    totalWritten += bytesRead;
                }
                return totalWritten;
            }
        } finally {
            CloseHandle(hDisk);
        }
    }
}
"@
    }

    try {
        $physPath = "\\.\PhysicalDrive$DiskNumber"
        $written = [RawDiskWriter]::WriteImage($physPath, $isoFullPath)
        Write-Host "Written $([math]::Round($written / 1MB, 1)) MB to disk $DiskNumber" -ForegroundColor Green
        $writeSuccess = $true
    } catch {
        Write-Host "Raw write failed: $_" -ForegroundColor Red
    }
} 

if (-not $writeSuccess) {
    # Fallback: try .NET without WSL
    if (-not ([System.Management.Automation.PSTypeName]'RawDiskWriter').Type) {
        Write-Host "Fallback also unavailable." -ForegroundColor Red
    } else {
        try {
            $physPath = "\\.\PhysicalDrive$DiskNumber"
            $written = [RawDiskWriter]::WriteImage($physPath, $isoFullPath)
            Write-Host "Written $([math]::Round($written / 1MB, 1)) MB to disk $DiskNumber" -ForegroundColor Green
            $writeSuccess = $true
        } catch {
            Write-Host "Fallback raw write also failed: $_" -ForegroundColor Red
        }
    }
}

if (-not $writeSuccess) {
    Write-Host "`nAll methods failed. Use Rufus (rufus.ie) to flash $IsoPath in DD mode." -ForegroundColor Yellow
    exit 1
}

Write-Host "[3/3] Syncing..." -ForegroundColor Yellow

# Force Windows to rescan the disk
$null = "intcaller" # small delay
Update-Disk -Number $DiskNumber -ErrorAction SilentlyContinue

Write-Host "`n=== USB FLASH COMPLETE ===" -ForegroundColor Green
Write-Host @"

TrustOS is now on disk $DiskNumber ($($targetDisk.FriendlyName))

To boot:
  1. Plug the USB into your target PC (with the RX 5600 XT)
  2. Enter BIOS/UEFI boot menu (usually F12 or F2 at startup)
  3. Select the USB drive (UEFI mode preferred)
  4. TrustOS will boot and the AMD GPU driver will probe the RX 5600 XT!

Note: Serial output goes to COM1 (115200 baud) if you have a serial adapter.
      Without serial, GPU detection results appear on the TrustOS desktop.
      Use the 'gpu info' command in the shell to see full GPU details.
"@ -ForegroundColor Cyan
