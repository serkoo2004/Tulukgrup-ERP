$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$realtime = Join-Path $root "realtime"
$go = Get-Command go -ErrorAction SilentlyContinue
$port = 8090

if (-not $go) {
    throw "go bulunamadi. Realtime servisi icin Go kurulumu gerekiyor."
}

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

if (Test-TcpPort -HostName "127.0.0.1" -Port $port) {
    Write-Host "Realtime zaten calisiyor: http://127.0.0.1:$port"
    exit 0
}

Start-Process `
    -FilePath $go.Source `
    -ArgumentList @("run", "./cmd/gateway") `
    -WorkingDirectory $realtime `
    -WindowStyle Hidden

for ($i = 0; $i -lt 60; $i++) {
    Start-Sleep -Milliseconds 500
    if (Test-TcpPort -HostName "127.0.0.1" -Port $port) {
        Write-Host "Realtime calisiyor: http://127.0.0.1:$port"
        exit 0
    }
}

throw "Realtime baslatildi ama port acilmadi: 127.0.0.1:$port"
