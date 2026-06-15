$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$mobile = Join-Path $root "mobile"

if (-not (Test-Path (Join-Path $mobile "package.json"))) {
    throw "Mobile package.json bulunamadi."
}

if (-not (Test-Path (Join-Path $mobile "node_modules"))) {
    Write-Host "[WARN] Mobile node_modules yok. Kontrol icin once: cd mobile; npm install"
    exit 0
}

Push-Location $mobile
try {
    npm run typecheck
} finally {
    Pop-Location
}
