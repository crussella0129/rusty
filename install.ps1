# Rusty installer & launcher script for Windows
# Paced, pedagogical bootstrap experience (prompt §0 / §7)

$ErrorActionPreference = "Stop"

Write-Host "========================================================================" -ForegroundColor Cyan
Write-Host "                      Rusty — Learn Rust by Doing                       " -ForegroundColor Cyan
Write-Host "========================================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Welcome! I am Rusty's setup guide."
Write-Host "The first lesson of learning Rust is compilation: you will build Rusty from"
Write-Host "source using your own local Rust compiler."
Write-Host ""

# 1. Detect Rust Toolchain
Write-Host "[1/3] Detecting Rust compiler toolchain..." -ForegroundColor Blue

$has_cargo = $null -ne (Get-Command cargo -ErrorAction SilentlyContinue)

if ($has_cargo) {
    $version = cargo --version
    Write-Host "✓ Found Rust toolchain: $version" -ForegroundColor Green
} else {
    Write-Host "⚠ Rust toolchain was not found on your system." -ForegroundColor Yellow
    Write-Host "To compile and run Rusty, we need to install the official Rust compiler."
    Write-Host ""
    
    $choice = Read-Host "Would you like to install Rust via rustup now? (y/N)"
    if ($choice -match '^[Yy]$') {
        Write-Host ""
        Write-Host "Downloading official Windows rustup installer..." -ForegroundColor Blue
        
        $temp_installer = Join-Path $env:TEMP "rustup-init.exe"
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $temp_installer
        
        Write-Host "Running rustup-init.exe..." -ForegroundColor Blue
        # Run rustup-init in default mode (-y) and wait for it
        Start-Process -FilePath $temp_installer -ArgumentList "-y" -Wait
        
        # Clean up
        Remove-Item $temp_installer -ErrorAction SilentlyContinue
        
        # Refresh current session path
        $env:Path += ";$env:USERPROFILE\.cargo\bin"
        
        if (Get-Command cargo -ErrorAction SilentlyContinue) {
            Write-Host "✓ Rust successfully installed!" -ForegroundColor Green
        } else {
            Write-Host "⚠ Rust was installed, but 'cargo' is not yet in your PATH." -ForegroundColor Yellow
            Write-Host "Please open a new PowerShell console, navigate back here, and run this script again."
            exit 0
        }
    } else {
        Write-Host ""
        Write-Host "Understood. You chose not to install Rust automatically."
        Write-Host "To continue, you can manually download and run the Rust installer from:"
        Write-Host "  https://rustup.rs/" -ForegroundColor DarkCyan
        Write-Host "After installing, please run this script again to build and launch Rusty."
        exit 0
    }
}

# 2. Build Rusty from Source
Write-Host ""
Write-Host "[2/3] Compiling Rusty from source..." -ForegroundColor Blue
Write-Host "Your toolchain will compile all crates in the workspace in release mode."
Write-Host "This might take a few minutes as Cargo builds and optimizes everything."
Write-Host "Enjoy watching the build process — this is the real compiler at work!"
Write-Host ""

cargo build --release

# 3. Launch Rusty
Write-Host ""
Write-Host "✓ Build complete!" -ForegroundColor Green
Write-Host "[3/3] Launching Rusty..." -ForegroundColor Blue

if (Test-Path ".\target\release\rusty.exe") {
    Start-Process ".\target\release\rusty.exe"
} else {
    Write-Host "⚠ Could not locate the built binary at '.\target\release\rusty.exe'." -ForegroundColor Yellow
    exit 1
}
