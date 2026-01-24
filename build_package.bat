@echo off
setlocal

REM Ensure we are in the project root (where this script resides)
cd /d "%~dp0"

echo [INFO] Starting clean build...

REM ------------------------------------------------------------------
REM Build Release
REM ------------------------------------------------------------------
cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Build failed.
    exit /b %errorlevel%
)

REM ------------------------------------------------------------------
REM Deploy to Root
REM ------------------------------------------------------------------
echo [INFO] Deploying artifacts to project root...

if not exist "target\release\typesafe.exe" (
    echo [ERROR] Build artifact not found in target\release\typesafe.exe
    exit /b 1
)

copy /y "target\release\typesafe.exe" ".\typesafe.exe" >nul

REM Copy PDFium if present (needed for runtime)
if exist "deps\pdfium.dll" (
    copy /y "deps\pdfium.dll" ".\pdfium.dll" >nul
)

REM Copy Tectonic if present (needed for runtime)
if exist "deps\tectonic.exe" (
    copy /y "deps\tectonic.exe" ".\tectonic.exe" >nul
)

REM ------------------------------------------------------------------
REM Cleanup
REM ------------------------------------------------------------------
echo [INFO] Cleaning up build artifacts...
if exist "target" rmdir /s /q "target"
if exist "dist" rmdir /s /q "dist"

echo [SUCCESS] Build complete. typesafe.exe is ready in the project root.
endlocal
