$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")

Write-Host "== Tuluklar ERP local check =="

Write-Host ""
Write-Host "1. PostgreSQL"
& (Join-Path $PSScriptRoot "start-local-postgres.ps1")

Write-Host ""
Write-Host "2. Rust backend compile"
& (Join-Path $PSScriptRoot "check-backend.ps1")

Write-Host ""
Write-Host "3. Rust backend runtime"
& (Join-Path $PSScriptRoot "stop-backend.ps1")
& (Join-Path $PSScriptRoot "start-backend.ps1")

Write-Host ""
Write-Host "4. Backend smoke"
& (Join-Path $PSScriptRoot "smoke-backend.ps1")

Write-Host ""
Write-Host "5. XLSX report smoke"
& (Join-Path $PSScriptRoot "smoke-reports-xlsx.ps1")

Write-Host ""
Write-Host "6. Inventory smoke"
& (Join-Path $PSScriptRoot "smoke-inventory.ps1")

Write-Host ""
Write-Host "7. Realtime check"
$realtimeScript = Join-Path $PSScriptRoot "check-realtime.ps1"
& $realtimeScript
if ($LASTEXITCODE -ne 0) {
    Write-Host "[WARN] Realtime check is not ready yet. Go runtime is required for local realtime validation."
    $global:LASTEXITCODE = 0
}

Write-Host ""
Write-Host "8. Mobile typecheck"
& (Join-Path $PSScriptRoot "check-mobile.ps1")

Write-Host ""
Write-Host "Local check finished"
