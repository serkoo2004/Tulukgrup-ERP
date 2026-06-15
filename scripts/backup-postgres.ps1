param(
  [string]$BackupDir = $env:BACKUP_DIR,
  [string]$DatabaseUrl = $env:DATABASE_URL,
  [string]$StorageRoot = $env:STORAGE_ROOT
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($BackupDir)) {
  $BackupDir = ".\backups"
}

if ([string]::IsNullOrWhiteSpace($DatabaseUrl)) {
  throw "DATABASE_URL is required. Set it in .env or pass -DatabaseUrl."
}

if ([string]::IsNullOrWhiteSpace($StorageRoot)) {
  $StorageRoot = ".\storage"
}

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$backupRoot = Join-Path $BackupDir "backup-$timestamp"
$dbDump = Join-Path $backupRoot "postgres.dump"
$storageBackup = Join-Path $backupRoot "storage"
$manifestPath = Join-Path $backupRoot "manifest.json"

New-Item -ItemType Directory -Force -Path $backupRoot | Out-Null

Write-Host "Creating PostgreSQL backup..."
& pg_dump --format=custom --file $dbDump $DatabaseUrl

if (Test-Path $StorageRoot) {
  Write-Host "Copying storage files..."
  Copy-Item -Path $StorageRoot -Destination $storageBackup -Recurse -Force
}

$manifest = [ordered]@{
  created_at = (Get-Date).ToUniversalTime().ToString("o")
  database_url_redacted = ($DatabaseUrl -replace '://([^:]+):([^@]+)@', '://$1:***@')
  db_dump = "postgres.dump"
  storage_included = (Test-Path $StorageRoot)
  app = "Tuluklar ERP"
}

$manifest | ConvertTo-Json -Depth 5 | Set-Content -Path $manifestPath -Encoding UTF8

$zipPath = "$backupRoot.zip"
if (Test-Path $zipPath) {
  Remove-Item -LiteralPath $zipPath -Force
}

Compress-Archive -LiteralPath $backupRoot -DestinationPath $zipPath -CompressionLevel Optimal
Write-Host "Backup created: $zipPath"
