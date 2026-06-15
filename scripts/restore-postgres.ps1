param(
  [Parameter(Mandatory = $true)]
  [string]$DumpPath,
  [string]$DatabaseUrl = $env:DATABASE_URL
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($DatabaseUrl)) {
  throw "DATABASE_URL is required. Set it in .env or pass -DatabaseUrl."
}

if (-not (Test-Path $DumpPath)) {
  throw "Dump file not found: $DumpPath"
}

Write-Host "Restoring PostgreSQL dump..."
& pg_restore --clean --if-exists --no-owner --dbname $DatabaseUrl $DumpPath
Write-Host "Restore completed."
