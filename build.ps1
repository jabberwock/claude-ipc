# Build script for Windows PowerShell

Write-Host "🔨 Building collab CLI and server..." -ForegroundColor Cyan

# Build CLI
Write-Host "Building CLI..." -ForegroundColor Yellow
Set-Location collab-cli
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ CLI build failed" -ForegroundColor Red
    exit 1
}
Set-Location ..

# Build Server
Write-Host "Building server..." -ForegroundColor Yellow
Set-Location collab-server
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Server build failed" -ForegroundColor Red
    exit 1
}
Set-Location ..

Write-Host "✓ Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "📦 Binaries located at:" -ForegroundColor Cyan
Write-Host "  CLI:    collab-cli\target\release\collab.exe"
Write-Host "  Server: collab-server\target\release\collab-server.exe"
Write-Host ""
Write-Host "To install system-wide:" -ForegroundColor Cyan
Write-Host "  mkdir -Force `$env:USERPROFILE\bin"
Write-Host "  copy collab-cli\target\release\collab.exe `$env:USERPROFILE\bin\"
Write-Host "  copy collab-server\target\release\collab-server.exe `$env:USERPROFILE\bin\"
Write-Host ""
Write-Host "To configure (add to `$PROFILE):" -ForegroundColor Cyan
Write-Host "  `$env:PATH = `"`$env:USERPROFILE\bin;`$env:PATH`""
Write-Host "  `$env:COLLAB_SERVER = 'http://localhost:8000'"
