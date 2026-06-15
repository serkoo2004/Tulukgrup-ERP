$ErrorActionPreference = "Stop"

Write-Host "Checking local ERP toolchain..."

$commands = @(
  @{ Name = "Rust compiler"; Command = "rustc"; Args = @("--version") },
  @{ Name = "Cargo"; Command = "cargo"; Args = @("--version") },
  @{ Name = "Go"; Command = "go"; Args = @("version") },
  @{ Name = "Docker"; Command = "docker"; Args = @("--version") }
)

foreach ($item in $commands) {
  try {
    $output = & $item.Command @($item.Args) 2>$null
    Write-Host "[OK] $($item.Name): $output"
  } catch {
    Write-Host "[MISSING] $($item.Name)"
  }
}

Write-Host ""
Write-Host "When all tools are installed:"
Write-Host "  Copy-Item .env.example .env"
Write-Host "  docker compose up -d postgres redis nats"
Write-Host "  cd backend; cargo run"
Write-Host "  cd ../realtime; go run ./cmd/gateway"
