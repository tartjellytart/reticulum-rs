# ESP32 Flashing Helper Script
# This script helps flash your ESP32 WROOM module

Write-Host "=== ESP32 Flashing Helper ===" -ForegroundColor Cyan
Write-Host ""

# Check if ESP environment is loaded
if (-not ($env:PATH -like "*espressif*")) {
    Write-Host "⚠️  ESP environment not loaded. Loading now..." -ForegroundColor Yellow
    . $HOME\export-esp.ps1
}

Write-Host "📋 Step 1: Put ESP32 in Bootloader Mode" -ForegroundColor Yellow
Write-Host ""
Write-Host "  1. Locate the BOOT button (GPIO0) on your ESP32 board"
Write-Host "  2. Locate the RESET button (EN or RST)"
Write-Host "  3. Hold the BOOT button down"
Write-Host "  4. While holding BOOT, press and release the RESET button"
Write-Host "  5. Release the BOOT button"
Write-Host "  6. The ESP32 is now in bootloader mode"
Write-Host ""
Write-Host "Press any key when you've completed these steps..." -ForegroundColor Green
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

Write-Host ""
Write-Host "🔍 Step 2: Checking connection..." -ForegroundColor Yellow
espflash board-info --chip esp32

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✅ Connection successful!" -ForegroundColor Green
    Write-Host ""
    Write-Host "📤 Step 3: Ready to flash" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To flash your project, run:" -ForegroundColor Cyan
    Write-Host "  cargo espflash flash --release --monitor" -ForegroundColor White
    Write-Host ""
    Write-Host "Or if using espflash directly:" -ForegroundColor Cyan
    Write-Host "  espflash flash target/xtensa-esp32-espidf/release/<your-binary>.elf" -ForegroundColor White
} else {
    Write-Host ""
    Write-Host "❌ Connection failed. Troubleshooting:" -ForegroundColor Red
    Write-Host ""
    Write-Host "  1. Make sure USB cable supports data (not charge-only)"
    Write-Host "  2. Try a different USB port"
    Write-Host "  3. Close any programs using COM3 (Arduino IDE, serial monitors)"
    Write-Host "  4. Try the bootloader sequence again"
    Write-Host "  5. Check Device Manager for driver issues"
    Write-Host ""
    Write-Host "Current COM port: COM3" -ForegroundColor Gray
}








