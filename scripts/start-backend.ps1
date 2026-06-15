$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$backend = Join-Path $root "backend"
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$stableBin = Join-Path $env:USERPROFILE ".rustup\toolchains\stable-x86_64-pc-windows-msvc\bin"
$vsDevCmd = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat"
$runner = Join-Path $PSScriptRoot "run-backend-local.cmd"
$port = 8080
$outLog = Join-Path $backend "backend.out.log"
$errLog = Join-Path $backend "backend.err.log"

function Test-TcpPort {
    param(
        [string]$HostName,
        [int]$Port
    )

    $client = [System.Net.Sockets.TcpClient]::new()
    try {
        $task = $client.ConnectAsync($HostName, $Port)
        if (-not $task.Wait(500)) {
            return $false
        }
        return $client.Connected
    } catch {
        return $false
    } finally {
        $client.Dispose()
    }
}

if (-not (Test-Path (Join-Path $cargoBin "cargo.exe"))) {
    throw "cargo.exe bulunamadi. Rustup kurulumu tamamlanmamis."
}

if (-not (Test-Path $vsDevCmd)) {
    throw "VsDevCmd.bat bulunamadi. Visual Studio Build Tools C++ workload gerekiyor."
}

if (-not (Test-Path $runner)) {
    throw "Backend runner bulunamadi: $runner"
}

$isOpen = Test-TcpPort -HostName "127.0.0.1" -Port $port
if ($isOpen) {
    Write-Host "Backend zaten calisiyor: http://127.0.0.1:$port"
    exit 0
}

Remove-Item -LiteralPath $outLog, $errLog -Force -ErrorAction SilentlyContinue

Start-Process `
    -FilePath $runner `
    -WorkingDirectory $backend `
    -WindowStyle Hidden

for ($i = 0; $i -lt 180; $i++) {
    Start-Sleep -Milliseconds 500
    $isOpen = Test-TcpPort -HostName "127.0.0.1" -Port $port
    if ($isOpen) {
        Write-Host "Backend calisiyor: http://127.0.0.1:$port"
        exit 0
    }
}

if (Test-Path $errLog) {
    Get-Content -Path $errLog -Tail 80
}

throw "Backend baslatildi ama port acilmadi: 127.0.0.1:$port"
