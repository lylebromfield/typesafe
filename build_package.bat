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

REM Generate icon from SVG if present
echo [INFO] Generating icon.png...
".\typesafe.exe" --gen-icon

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

REM Create distribution archive
echo [INFO] Creating distribution archive...
if exist "release_dist" rmdir /s /q "release_dist"
mkdir "release_dist"
mkdir "release_dist\web"

echo Copying release files...
copy /y "target\release\typesafe.exe" "release_dist\"
if exist "deps\tectonic.exe" copy /y "deps\tectonic.exe" "release_dist\"
if exist "deps\pdfium.dll" copy /y "deps\pdfium.dll" "release_dist\"
if exist "dictionary.txt" copy /y "dictionary.txt" "release_dist\"
if exist "web\logo.svg" copy /y "web\logo.svg" "release_dist\web\"
if exist "icon.png" copy /y "icon.png" "release_dist\"

echo Zipping release...
powershell Compress-Archive -Path "release_dist\*" -DestinationPath "typesafe_v0.2.0.zip" -Force

echo [SUCCESS] Build complete. Archive created at typesafe_v0.2.0.zip
endlocal
