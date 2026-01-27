@echo off
setlocal

REM Ensure we are in the project root (where this script resides)
cd /d "%~dp0"

echo [INFO] Starting build...

REM ------------------------------------------------------------------
REM Generate Data
REM ------------------------------------------------------------------
echo [INFO] Generating latex_data.json...
cargo run --release --bin ingest_cwl
if %errorlevel% neq 0 (
    echo [ERROR] Data generation failed.
    exit /b %errorlevel%
)

REM ------------------------------------------------------------------
REM Build Release
REM ------------------------------------------------------------------
cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Build failed.
    exit /b %errorlevel%
)

if not exist "target\release\typesafe.exe" (
    echo [ERROR] Build artifact not found in target\release\typesafe.exe
    exit /b 1
)

REM ------------------------------------------------------------------
REM Populate target/release (Standalone Run Support)
REM ------------------------------------------------------------------
echo [INFO] Populating target/release for standalone running...

if exist "deps\install-tl-windows.exe" (
    echo Copying install-tl-windows.exe to target/release...
    copy /y "deps\install-tl-windows.exe" "target\release\" >nul
)
if exist "deps\pdfium.dll" (
    echo Copying pdfium.dll to target/release...
    copy /y "deps\pdfium.dll" "target\release\" >nul
)
if exist "icon.png" (
    echo Copying icon.png to target/release...
    copy /y "icon.png" "target\release\" >nul
)
if exist "dictionary.txt" (
    echo Copying dictionary.txt to target/release...
    copy /y "dictionary.txt" "target\release\" >nul
)
if exist "latex_data.json" (
    echo Copying latex_data.json to target/release...
    copy /y "latex_data.json" "target\release\" >nul
)

REM ------------------------------------------------------------------
REM Create Distribution Archive (for Web)
REM ------------------------------------------------------------------
echo [INFO] Creating distribution archive...

if exist "release_dist" rmdir /s /q "release_dist"
mkdir "release_dist"
mkdir "release_dist\web"

echo Copying release files...
copy /y "target\release\typesafe.exe" "release_dist\" >nul
if exist "deps\install-tl-windows.exe" copy /y "deps\install-tl-windows.exe" "release_dist\" >nul
if exist "deps\pdfium.dll" copy /y "deps\pdfium.dll" "release_dist\" >nul
if exist "dictionary.txt" copy /y "dictionary.txt" "release_dist\" >nul
if exist "latex_data.json" copy /y "latex_data.json" "release_dist\" >nul
if exist "web\logo.svg" copy /y "web\logo.svg" "release_dist\web\" >nul
if exist "icon.png" copy /y "icon.png" "release_dist\" >nul

echo Zipping release...
powershell Compress-Archive -Path "release_dist\*" -DestinationPath "typesafe-alpha.zip" -Force

echo Cleaning up temporary files...
if exist "release_dist" rmdir /s /q "release_dist"

echo [SUCCESS] Build complete.
echo - Binary: target\release\typesafe.exe (Standalone)
echo - Archive: typesafe-alpha.zip
endlocal
