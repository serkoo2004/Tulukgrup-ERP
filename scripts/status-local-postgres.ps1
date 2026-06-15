$ErrorActionPreference = "Stop"

$pgBin = "C:\Program Files\PostgreSQL\18\bin"
$psql = Join-Path $pgBin "psql.exe"
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

$isOpen = Test-TcpPort -HostName "127.0.0.1" -Port $port
if (-not $isOpen) {
    Write-Host "PostgreSQL kapali: 127.0.0.1:$port"
    exit 0
}

Write-Host "PostgreSQL acik: 127.0.0.1:$port"
if (Test-Path $psql) {
    & $psql -h 127.0.0.1 -p $port -U postgres -d tuluklar_erp -c "SELECT current_database(), current_user, inet_server_port();"
}
