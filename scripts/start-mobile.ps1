$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$mobile = Join-Path $root "mobile"

if (-not (Test-Path (Join-Path $mobile "package.json"))) {
    throw "Mobile package.json bulunamadi: $mobile"
}

if (-not (Test-Path (Join-Path $mobile "node_modules"))) {
    throw "Mobile node_modules bulunamadi. Once `cd mobile; npm install` calistir."
}

if (-not $env:EXPO_PUBLIC_API_BASE_URL) {
    Write-Host "EXPO_PUBLIC_API_BASE_URL tanimli degil. Varsayilan: http://127.0.0.1:8080"
    Write-Host "Telefon/Tailscale testinde ornek: `$env:EXPO_PUBLIC_API_BASE_URL='http://100.95.238.26:8080'"
}

Push-Location $mobile
try {
    npm run start
} finally {
    Pop-Location
}
