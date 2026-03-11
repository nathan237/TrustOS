param(
    [string]$IsoPath = "$PSScriptRoot\trustos.iso",
    [string]$Command = "",
    [int]$SerialPort = 4444
)

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
if (-not (Test-Path $QemuExe)) {
    Write-Error "QEMU not found at $QemuExe"
    exit 1
}
if (-not (Test-Path $IsoPath)) {
    Write-Error "ISO not found: $IsoPath"
    exit 1
}

$serialArg = "tcp:127.0.0.1:$SerialPort,server,nowait"
$arguments = @(
    "-cdrom", "`"$IsoPath`"",
    "-m", "128M",
    "-display", "gtk",
    "-vga", "std",
    "-device", "virtio-net-pci,netdev=net0",
    "-netdev", "user,id=net0",
    "-serial", $serialArg,
    "-no-reboot"
)

Start-Process -FilePath $QemuExe -ArgumentList $arguments | Out-Null

$client = New-Object System.Net.Sockets.TcpClient
$connected = $false
for ($i = 0; $i -lt 50; $i++) {
    try {
        $client.Connect("127.0.0.1", $SerialPort)
        $connected = $true
        break
    } catch {
        Start-Sleep -Milliseconds 200
    }
}

if (-not $connected) {
    Write-Error "Could not connect to serial TCP port $SerialPort"
    exit 1
}

$stream = $client.GetStream()
$stream.ReadTimeout = 2000
$writer = New-Object System.IO.StreamWriter($stream)
$writer.AutoFlush = $true

$writer.WriteLine($Command)

$buffer = New-Object byte[] 4096
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 10) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) {
            $text = [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            Write-Host $text -NoNewline
        }
    } else {
        Start-Sleep -Milliseconds 100
    }
}

$client.Close()
