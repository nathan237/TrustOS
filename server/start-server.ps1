# Simple HTTP Server for TrustOS packages
# Run this script AS ADMINISTRATOR to start a local server on port 8080

$port = 8080
$root = Join-Path $PSScriptRoot "packages"

Write-Host "Starting TrustOS Package Server on http://0.0.0.0:$port" -ForegroundColor Green
Write-Host "Serving files from: $root" -ForegroundColor Cyan
Write-Host "VM should connect to: 10.0.2.2:$port" -ForegroundColor Yellow
Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""

$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://*:$port/")
$listener.Start()

Write-Host "Server running..." -ForegroundColor Green

try {
    while ($listener.IsListening) {
        $context = $listener.GetContext()
        $request = $context.Request
        $response = $context.Response
        
        $path = $request.Url.LocalPath.TrimStart('/')
        $filePath = Join-Path $root $path
        
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] $($request.HttpMethod) /$path" -ForegroundColor White
        
        if (Test-Path $filePath -PathType Leaf) {
            $bytes = [System.IO.File]::ReadAllBytes($filePath)
            $response.ContentLength64 = $bytes.Length
            $response.ContentType = "application/octet-stream"
            $response.OutputStream.Write($bytes, 0, $bytes.Length)
            Write-Host "  -> 200 OK ($($bytes.Length) bytes)" -ForegroundColor Green
        } else {
            $response.StatusCode = 404
            $msg = [System.Text.Encoding]::UTF8.GetBytes("Not Found: $path")
            $response.OutputStream.Write($msg, 0, $msg.Length)
            Write-Host "  -> 404 Not Found" -ForegroundColor Red
        }
        
        $response.Close()
    }
} finally {
    $listener.Stop()
    Write-Host "Server stopped" -ForegroundColor Yellow
}
