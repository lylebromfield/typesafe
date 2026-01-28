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
REM Code Sign the Executable (Optional - requires certificate)
REM ------------------------------------------------------------------
if exist "typesafe.pfx" (
    echo [INFO] Signing executable...
    REM Set CODE_SIGN_PASSWORD environment variable before running this script
    REM Example: set CODE_SIGN_PASSWORD=your_certificate_password
    if defined CODE_SIGN_PASSWORD (
        signtool sign /f typesafe.pfx /p "%CODE_SIGN_PASSWORD%" /fd SHA256 /tr http://timestamp.sectigo.com /td SHA256 "target\release\typesafe.exe"
        if %errorlevel% neq 0 (
            echo [WARNING] Code signing failed. Continuing without signature.
        ) else (
            echo [SUCCESS] Executable signed successfully.
        )
    ) else (
        echo [WARNING] CODE_SIGN_PASSWORD not set. Skipping code signing.
        echo [INFO] To enable signing, set: set CODE_SIGN_PASSWORD=your_password
    )
) else (
    echo [INFO] No code-signing certificate found (typesafe.pfx). Skipping signing.
    echo [INFO] See SIGNING_GUIDE.md for instructions on obtaining and using a certificate.
)

REM ------------------------------------------------------------------
REM Populate target/release (Standalone Run Support)
REM ------------------------------------------------------------------
echo [INFO] Populating target/release for standalone running...


if exist "deps\tectonic.exe" (
    echo Copying tectonic.exe to target/release...
    copy /y "deps\tectonic.exe" "target\release\" >nul
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
if exist "deps\tectonic.exe" copy /y "deps\tectonic.exe" "release_dist\" >nul

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
