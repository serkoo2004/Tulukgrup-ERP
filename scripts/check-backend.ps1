$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$backend = Join-Path $root "backend"
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$stableBin = Join-Path $env:USERPROFILE ".rustup\toolchains\stable-x86_64-pc-windows-msvc\bin"
$vsDevCmd = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat"

if (-not (Test-Path (Join-Path $cargoBin "cargo.exe"))) {
    throw "cargo.exe bulunamadi. Rustup kurulumu tamamlanmamis."
}

if (-not (Test-Path $vsDevCmd)) {
    throw "VsDevCmd.bat bulunamadi. Visual Studio Build Tools C++ workload gerekiyor."
}

$command = "`"$vsDevCmd`" -arch=x64 && set PATH=$stableBin;$cargoBin;%PATH% && cargo check"
Push-Location $backend
try {
    & cmd.exe /d /s /c $command
} finally {
    Pop-Location
}
