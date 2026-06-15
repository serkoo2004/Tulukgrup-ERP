$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$pgBin = "C:\Program Files\PostgreSQL\18\bin"
$postgres = Join-Path $pgBin "postgres.exe"
$dataDir = Join-Path $root "database\local-postgres-data"
$port = 5432

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

if (-not (Test-Path $postgres)) {
    throw "postgres.exe bulunamadi: $postgres"
}

if (-not (Test-Path $dataDir)) {
    throw "Lokal PostgreSQL data klasoru yok: $dataDir"
}

$isOpen = Test-TcpPort -HostName "127.0.0.1" -Port $port
if ($isOpen) {
    Write-Host "PostgreSQL zaten calisiyor: 127.0.0.1:$port"
    exit 0
}

Start-Process `
    -FilePath $postgres `
    -ArgumentList "-D `"$dataDir`" -p $port" `
    -WorkingDirectory $root `
    -WindowStyle Hidden

Start-Sleep -Seconds 2
$isOpen = Test-TcpPort -HostName "127.0.0.1" -Port $port
if (-not $isOpen) {
    throw "PostgreSQL baslatildi ama port acilmadi: 127.0.0.1:$port"
}

Write-Host "PostgreSQL calisiyor: 127.0.0.1:$port"
