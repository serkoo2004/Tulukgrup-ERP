$ErrorActionPreference = "Stop"

$port = 5173
$lines = netstat -ano -p tcp | Select-String -Pattern ":$port\s+.*LISTENING\s+(\d+)"

if (-not $lines) {
    Write-Host "Frontend kapali: http://127.0.0.1:$port"
    exit 0
}

$processIds = $lines | ForEach-Object { [regex]::Match($_.Line, "LISTENING\s+(\d+)").Groups[1].Value } | Sort-Object -Unique
foreach ($processId in $processIds) {
    Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
}

Write-Host "Frontend durduruldu: http://127.0.0.1:$port"
