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

# ======== Method 1: Direct raw write (dd-style) ========
Write-Host "`n[1/3] Cleaning disk..." -ForegroundColor Yellow

# Clear the disk
Clear-Disk -Number $DiskNumber -RemoveData -RemoveOEM -Confirm:$false -ErrorAction SilentlyContinue
Set-Disk -Number $DiskNumber -IsOffline $false -ErrorAction SilentlyContinue
Set-Disk -Number $DiskNumber -IsReadOnly $false -ErrorAction SilentlyContinue

Write-Host "[2/3] Writing ISO to USB (raw dd-style)..." -ForegroundColor Yellow

# Direct raw write using .NET - this writes the ISO image byte-by-byte to the disk
# This preserves the hybrid ISO structure (BIOS + UEFI bootable)
$physPath = "\\.\PhysicalDrive$DiskNumber"

try {
    # Open the ISO file for reading
    $isoStream = [System.IO.File]::OpenRead((Resolve-Path $IsoPath).Path)
    
    # Open the physical disk for writing
    # Using .NET P/Invoke for raw disk access
    Add-Type -TypeDefinition @"
using System;
using System.IO;
using System.Runtime.InteropServices;

public class RawDisk {
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    static extern IntPtr CreateFile(
        string lpFileName,
        uint dwDesiredAccess,
        uint dwShareMode,
        IntPtr lpSecurityAttributes,
        uint dwCreationDisposition,
        uint dwFlagsAndAttributes,
        IntPtr hTemplateFile);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool WriteFile(
        IntPtr hFile,
        byte[] lpBuffer,
        uint nNumberOfBytesToWrite,
        out uint lpNumberOfBytesWritten,
        IntPtr lpOverlapped);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool CloseHandle(IntPtr hObject);

    const uint GENERIC_WRITE = 0x40000000;
    const uint FILE_SHARE_READ = 0x00000001;
    const uint FILE_SHARE_WRITE = 0x00000002;
    const uint OPEN_EXISTING = 3;
    const uint FILE_FLAG_WRITE_THROUGH = 0x80000000;
    const uint FILE_FLAG_NO_BUFFERING = 0x20000000;

    public static long WriteImage(string diskPath, string isoPath) {
        IntPtr hDisk = CreateFile(
            diskPath,
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            IntPtr.Zero,
            OPEN_EXISTING,
            FILE_FLAG_WRITE_THROUGH | FILE_FLAG_NO_BUFFERING,
            IntPtr.Zero);

        if (hDisk == new IntPtr(-1)) {
            throw new Exception("Failed to open disk: " + Marshal.GetLastWin32Error());
        }

        try {
            using (var fs = File.OpenRead(isoPath)) {
                byte[] buffer = new byte[1048576]; // 1MB buffer
                long totalWritten = 0;
                int bytesRead;

                while ((bytesRead = fs.Read(buffer, 0, buffer.Length)) > 0) {
                    // Pad to 512-byte sector boundary
                    int aligned = ((bytesRead + 511) / 512) * 512;
                    if (aligned > bytesRead) {
                        Array.Clear(buffer, bytesRead, aligned - bytesRead);
                    }

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
"@ -ErrorAction SilentlyContinue

    $written = [RawDisk]::WriteImage($physPath, (Resolve-Path $IsoPath).Path)
    Write-Host "Written $([math]::Round($written / 1MB, 1)) MB to disk $DiskNumber" -ForegroundColor Green

} catch {
    Write-Host "Raw write failed: $_" -ForegroundColor Red
    Write-Host "`nFallback: Use Rufus (https://rufus.ie) to flash $IsoPath in DD mode." -ForegroundColor Yellow
    $isoStream.Close() 2>$null
    exit 1
} finally {
    if ($isoStream) { $isoStream.Close() }
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
