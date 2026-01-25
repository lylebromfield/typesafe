@echo off
setlocal

REM Ensure we are in the project root (where this script resides)
cd /d "%~dp0"

echo [INFO] Starting build...

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
    echo [INFO] Copying pdfium.dll to root...
    copy /y "deps\pdfium.dll" ".\pdfium.dll"
)

REM Copy executable to root
echo [INFO] Copying typesafe.exe to root...
copy /y "target\release\typesafe.exe" ".\typesafe.exe"

REM Copy Tectonic and Pdfium to target/release for standalone running
echo [INFO] Populating target/release...
if exist "deps\tectonic.exe" (
    echo Copying tectonic.exe to target/release...
    copy /y "deps\tectonic.exe" "target\release\"
)
if exist "deps\pdfium.dll" (
    echo Copying pdfium.dll to target/release...
    copy /y "deps\pdfium.dll" "target\release\"
)
if exist "icon.png" (
    echo Copying icon.png to target/release...
    copy /y "icon.png" "target\release\"
)
if exist "dictionary.txt" (
    echo Copying dictionary.txt to target/release...
    copy /y "dictionary.txt" "target\release\"
)

REM Copy dependencies to root for running from root
echo [INFO] Populating root...
if exist "deps\tectonic.exe" (
    echo Copying tectonic.exe to root...
    copy /y "deps\tectonic.exe" ".\tectonic.exe"
)
if exist "deps\pdfium.dll" (
    echo Copying pdfium.dll to root...
    copy /y "deps\pdfium.dll" ".\pdfium.dll"
)

echo [SUCCESS] Build complete. Executable available in project root and target/release.
endlocal
