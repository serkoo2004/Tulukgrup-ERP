$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$frontend = Join-Path $root "frontend"
$port = 5173
$outLog = Join-Path $frontend "vite.out.log"
$errLog = Join-Path $frontend "vite.err.log"

function Test-TcpPort {
    param(
        [string]$HostName,
        [int]$Port
    )

    $client = [System.Net.Sockets.TcpClient]::new()
    try {
        $task = $client.ConnectAsync($HostName, $Port)
        if (-not $task.Wait(500)) {
            return $false
        }
        return $client.Connected
    } catch {
        return $false
    } finally {
        $client.Dispose()
    }
}

if (-not (Test-Path (Join-Path $frontend "package.json"))) {
    throw "Frontend package.json bulunamadi: $frontend"
}

if (-not (Test-Path (Join-Path $frontend "node_modules"))) {
    throw "node_modules bulunamadi. Once frontend klasorunde npm install calistir."
}

if (Test-TcpPort -HostName "127.0.0.1" -Port $port) {
    Write-Host "Frontend zaten calisiyor: http://127.0.0.1:$port"
    Write-Host "LAN/Tailscale icin Vite logundaki Network adreslerini kontrol et: $outLog"
    exit 0
}

Remove-Item -LiteralPath $outLog, $errLog -Force -ErrorAction SilentlyContinue

Start-Process `
    -FilePath "npm.cmd" `
    -ArgumentList "run", "dev" `
    -WorkingDirectory $frontend `
    -WindowStyle Hidden `
    -RedirectStandardOutput $outLog `
    -RedirectStandardError $errLog

for ($i = 0; $i -lt 30; $i++) {
    Start-Sleep -Milliseconds 500
    if (Test-TcpPort -HostName "127.0.0.1" -Port $port) {
        Write-Host "Frontend calisiyor: http://127.0.0.1:$port"
        if (Test-Path $outLog) {
            Get-Content $outLog -Tail 20
        }
        exit 0
    }
}

if (Test-Path $errLog) {
    Get-Content $errLog -Tail 80
}

throw "Frontend baslatildi ama port acilmadi: 127.0.0.1:$port"
