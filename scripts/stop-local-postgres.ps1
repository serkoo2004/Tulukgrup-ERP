$ErrorActionPreference = "Stop"

$pgBin = "C:\Program Files\PostgreSQL\18\bin"
$psql = Join-Path $pgBin "psql.exe"
$port = 5432

if (-not (Test-Path $psql)) {
    throw "psql.exe bulunamadi: $psql"
}

$isOpen = (Test-NetConnection -ComputerName 127.0.0.1 -Port $port -InformationLevel Quiet)
if (-not $isOpen) {
    Write-Host "PostgreSQL zaten kapali: 127.0.0.1:$port"
    exit 0
}

& $psql -h 127.0.0.1 -p $port -U postgres -d postgres -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid();" | Out-Null
& $psql -h 127.0.0.1 -p $port -U postgres -d postgres -c "SELECT pg_reload_conf();" | Out-Null

$processes = Get-Process postgres -ErrorAction SilentlyContinue
foreach ($process in $processes) {
    Stop-Process -Id $process.Id -Force
}

Start-Sleep -Seconds 1
$isOpen = (Test-NetConnection -ComputerName 127.0.0.1 -Port $port -InformationLevel Quiet)
if ($isOpen) {
    throw "PostgreSQL durdurulamadi: 127.0.0.1:$port"
}

Write-Host "PostgreSQL durduruldu: 127.0.0.1:$port"
