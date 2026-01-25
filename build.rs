#[cfg(windows)]
extern crate winres;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        if std::path::Path::new("icon.ico").exists() {
            res.set_icon("icon.ico");
        }
        res.compile().unwrap();
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icon.ico");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    // Get the target directory
    let target_dir = out_path
        .ancestors()
        .find(|p| p.file_name().map_or(false, |n| n == "target"))
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let deps_dir = if let Some(parent) = target_dir.parent() {
        parent.to_path_buf().join("deps")
    } else {
        PathBuf::from("deps")
    };

    // Create deps directory
    let _ = fs::create_dir_all(&deps_dir);

    let pdfium_path = deps_dir.join("pdfium.dll");

    // Only download if pdfium.dll doesn't exist
    if !pdfium_path.exists() {
        println!("cargo:warning=Downloading pdfium.dll...");

        if let Err(e) = download_pdfium(&deps_dir) {
            println!("cargo:warning=Failed to download pdfium: {}", e);
            println!("cargo:warning=Please manually download from: https://github.com/bblanchon/pdfium-binaries/releases");
        }
    }

    let tectonic_path = deps_dir.join("tectonic.exe");
    if !tectonic_path.exists() {
        println!("cargo:warning=Downloading tectonic.exe...");
        if let Err(e) = download_tectonic(&deps_dir) {
            println!("cargo:warning=Failed to download tectonic: {}", e);
        }
    }

    // Tell cargo to link the library
    println!("cargo:rustc-link-search=native={}", deps_dir.display());
}

fn download_pdfium(deps_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let tgz_path = deps_dir.join("pdfium-temp.tgz");
    let url = "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7643/pdfium-win-x64.tgz";

    // Try using PowerShell on Windows
    #[cfg(target_os = "windows")]
    {
        let download_cmd = format!(
            "$ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri '{}' -OutFile '{}'",
            url, tgz_path.display()
        );

        let output = Command::new("powershell")
            .args(&["-Command", &download_cmd])
            .output()?;

        if !output.status.success() {
            return Err("PowerShell download failed".into());
        }

        // Extract the TGZ file using tar command (available on Windows 10+)
        let extract_output = Command::new("tar")
            .args(&["-xzf", tgz_path.to_str().unwrap(), "-C", deps_dir.to_str().unwrap()])
            .output()?;

        if !extract_output.status.success() {
            return Err("Tar extraction failed".into());
        }

        // Copy pdfium.dll from bin to deps root
        let bin_dll = deps_dir.join("bin").join("pdfium.dll");
        let deps_dll = deps_dir.join("pdfium.dll");

        if bin_dll.exists() {
            fs::copy(&bin_dll, &deps_dll)?;
        }

        // Clean up
        let _ = fs::remove_dir_all(deps_dir.join("bin"));
        let _ = fs::remove_dir_all(deps_dir.join("include"));
        let _ = fs::remove_dir_all(deps_dir.join("lib"));
        let _ = fs::remove_dir_all(deps_dir.join("licenses"));
        let _ = fs::remove_file(&tgz_path);
        let _ = fs::remove_file(deps_dir.join("LICENSE"));
        let _ = fs::remove_file(deps_dir.join("VERSION"));
        let _ = fs::remove_file(deps_dir.join("PDFiumConfig.cmake"));
        let _ = fs::remove_file(deps_dir.join("args.gn"));

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("pdfium download is currently only supported on Windows".into())
    }
}

fn download_tectonic(deps_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let zip_path = deps_dir.join("tectonic.zip");
    // Using a fixed version (0.15.0)
    let url = "https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic@0.15.0/tectonic-0.15.0-x86_64-pc-windows-msvc.zip";

    #[cfg(target_os = "windows")]
    {
        let download_cmd = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; $ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri '{}' -OutFile '{}'",
            url, zip_path.display()
        );

        let output = Command::new("powershell")
            .args(&["-Command", &download_cmd])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("PowerShell download failed: {}", stderr).into());
        }

        let unzip_cmd = format!(
            "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
            zip_path.display(), deps_dir.display()
        );

        let extract_output = Command::new("powershell")
            .args(&["-Command", &unzip_cmd])
            .output()?;

        if !extract_output.status.success() {
             return Err("Unzip failed".into());
        }

        let _ = fs::remove_file(&zip_path);
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("tectonic download is currently only supported on Windows".into())
    }
}
