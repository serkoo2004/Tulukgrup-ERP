$ErrorActionPreference = "Stop"

$go = Get-Command go -ErrorAction SilentlyContinue
if (-not $go) {
    Write-Host "[MISSING] Go runtime. Install Go before running realtime checks."
    exit 1
}

Push-Location (Join-Path (Resolve-Path (Join-Path $PSScriptRoot "..")).Path "realtime")
try {
    go version
    go test ./...
} finally {
    Pop-Location
}
