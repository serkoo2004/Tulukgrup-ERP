$ErrorActionPreference = "Stop"

$baseUrl = $env:BACKEND_BASE_URL
if (-not $baseUrl) {
    $baseUrl = "http://127.0.0.1:8080"
}

$email = $env:BOOTSTRAP_ADMIN_EMAIL
if (-not $email) {
    $email = "admin@tuluklar.local"
}

$password = $env:BOOTSTRAP_ADMIN_PASSWORD
if (-not $password) {
    $password = "Admin12345!"
}

Write-Host "Smoke test XLSX reports: $baseUrl"

$login = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/auth/login" `
    -ContentType "application/json" `
    -Body (@{
        email = $email
        password = $password
    } | ConvertTo-Json)

$headers = @{ Authorization = "Bearer $($login.access_token)" }
$reports = @(
    "management",
    "vehicles",
    "maintenances",
    "insurance-policies",
    "expenses",
    "damages",
    "vehicle-inspections",
    "value-loss-claims",
    "fuel-entries",
    "vehicle-washes",
    "insurance-quotes",
    "inventory-products",
    "inventory-movements",
    "inventory-critical",
    "support-tickets",
    "support-knowledge-base"
)
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) "tuluklar-xlsx-smoke"
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

foreach ($report in $reports) {
    $file = Join-Path $tempDir "$report.xlsx"
    Invoke-WebRequest `
        -Uri "$baseUrl/api/v1/reports/$report.xlsx" `
        -Headers $headers `
        -OutFile $file | Out-Null

    $bytes = [System.IO.File]::ReadAllBytes($file)
    if ($bytes.Length -lt 1000) {
        throw "$report.xlsx looks too small"
    }

    $signature = -join ($bytes[0..3] | ForEach-Object { $_.ToString("X2") })
    if ($signature -ne "504B0304") {
        throw "$report.xlsx is not a valid XLSX/ZIP signature: $signature"
    }

    Remove-Item -LiteralPath $file -Force
    Write-Host "[OK] $report.xlsx"
}

$spec = Invoke-RestMethod -Uri "$baseUrl/openapi.json"
if (-not $spec.paths.'/api/v1/reports/management.xlsx') {
    throw "OpenAPI does not expose management.xlsx"
}

if (-not $spec.paths.'/api/v1/reports/vehicles.xlsx') {
    throw "OpenAPI does not expose vehicles.xlsx"
}

if (-not $spec.paths.'/api/v1/reports/inventory-products.xlsx') {
    throw "OpenAPI does not expose inventory-products.xlsx"
}

if (-not $spec.paths.'/api/v1/reports/support-tickets.xlsx') {
    throw "OpenAPI does not expose support-tickets.xlsx"
}

if (-not $spec.paths.'/api/v1/reports/support-knowledge-base.xlsx') {
    throw "OpenAPI does not expose support-knowledge-base.xlsx"
}

if (-not $spec.paths.'/api/v1/inventory/products') {
    throw "OpenAPI does not expose inventory products API"
}

if ($spec.paths.'/api/v1/reports/vehicles.csv') {
    throw "OpenAPI still exposes CSV reports"
}

Write-Host "XLSX report smoke test passed"
