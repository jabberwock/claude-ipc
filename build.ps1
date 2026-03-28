# Build script for Windows PowerShell

Write-Host "Building collab CLI and server..." -ForegroundColor Cyan

# Build CLI
Write-Host "Building CLI..." -ForegroundColor Yellow
Set-Location collab-cli
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "CLI build failed" -ForegroundColor Red
    exit 1
}
Set-Location ..

# Build Server
Write-Host "Building server..." -ForegroundColor Yellow
Set-Location collab-server
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Server build failed" -ForegroundColor Red
    exit 1
}
Set-Location ..

Write-Host "Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Binaries:" -ForegroundColor Cyan
Write-Host "  CLI:    collab-cli\target\release\collab.exe"
Write-Host "  Server: collab-server\target\release\collab-server.exe"
Write-Host ""
Write-Host "Config: create %USERPROFILE%\.collab.toml with:" -ForegroundColor Cyan
Write-Host "  host = `"http://your-server:8000`""
Write-Host "  instance = `"your-worker-name`""
Write-Host "  recipients = [`"other-worker`"]"
