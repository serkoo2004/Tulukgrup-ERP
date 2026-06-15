$ErrorActionPreference = "Stop"

$port = 5173
$lines = netstat -ano -p tcp | Select-String -Pattern ":$port\s+.*LISTENING\s+(\d+)"

if (-not $lines) {
    Write-Host "Frontend kapali: http://127.0.0.1:$port"
    exit 0
}

Write-Host "Frontend calisiyor:"
$lines | ForEach-Object { Write-Host $_.Line }

$log = Join-Path (Resolve-Path (Join-Path $PSScriptRoot "..")).Path "frontend\vite.out.log"
if (Test-Path $log) {
    Write-Host ""
    Write-Host "Vite adresleri:"
    Get-Content $log -Tail 30
}
