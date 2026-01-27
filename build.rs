#[cfg(windows)]
extern crate winres;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        if Path::new("icon.ico").exists() {
            res.set_icon("icon.ico");
        }
        res.compile().unwrap();
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icon.ico");
    println!("cargo:rerun-if-changed=latex_data.json");

    // Get the project root directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root_dir = PathBuf::from(manifest_dir);
    let deps_dir = root_dir.join("deps");

    // Create deps directory
    let _ = fs::create_dir_all(&deps_dir);

    // Determine filenames based on OS
    let (pdfium_file, texlive_installer) = if cfg!(windows) {
        ("pdfium.dll", "install-tl-windows.exe")
    } else if cfg!(target_os = "macos") {
        ("libpdfium.dylib", "install-tl-unx.tar.gz")
    } else {
        ("libpdfium.so", "install-tl-unx.tar.gz")
    };

    let pdfium_path = deps_dir.join(pdfium_file);
    if !pdfium_path.exists() {
        println!("cargo:warning=Downloading {}...", pdfium_file);
        if let Err(e) = download_pdfium(&deps_dir, pdfium_file) {
            println!("cargo:warning=Failed to download pdfium: {}", e);
        }
    }

    let texlive_path = deps_dir.join(texlive_installer);
    if !texlive_path.exists() {
        println!("cargo:warning=Downloading TeX Live installer...");
        if let Err(e) = download_texlive(&deps_dir, texlive_installer) {
            println!("cargo:warning=Failed to download TeX Live installer: {}", e);
        }
    }

    // Tell cargo where to find native libraries
    println!("cargo:rustc-link-search=native={}", deps_dir.display());
}

fn download_pdfium(deps_dir: &Path, target_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let version = "7643";
    let url = if cfg!(windows) {
        format!("https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/{}/pdfium-win-x64.tgz", version)
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            format!("https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/{}/pdfium-mac-arm64.tgz", version)
        } else {
            format!("https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/{}/pdfium-mac-x64.tgz", version)
        }
    } else {
        format!("https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/{}/pdfium-linux-x64.tgz", version)
    };

    let archive_path = deps_dir.join("pdfium.tgz");
    download_file(&url, &archive_path)?;

    // Extract
    let mut cmd = Command::new("tar");
    cmd.args(&["-xzf", archive_path.to_str().unwrap(), "-C", deps_dir.to_str().unwrap()]);
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(format!("Extraction failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    // Move library to root of deps folder
    let subdirs = ["bin", "lib"];
    for subdir in subdirs {
        let candidate = deps_dir.join(subdir).join(target_file);
        if candidate.exists() {
            fs::copy(&candidate, deps_dir.join(target_file))?;
            break;
        }
    }

    // Cleanup
    let _ = fs::remove_file(&archive_path);
    let _ = fs::remove_dir_all(deps_dir.join("bin")).ok();
    let _ = fs::remove_dir_all(deps_dir.join("lib")).ok();
    let _ = fs::remove_dir_all(deps_dir.join("include")).ok();

    Ok(())
}



fn download_texlive(deps_dir: &Path, installer_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = if cfg!(windows) {
        "https://mirror.ctan.org/systems/texlive/tlnet/install-tl-windows.exe"
    } else {
        "https://mirror.ctan.org/systems/texlive/tlnet/install-tl-unx.tar.gz"
    };

    let archive_path = deps_dir.join(installer_file);
    download_file(url, &archive_path)?;

    // For tar.gz on Unix, extract it so it's ready to run
    if !cfg!(windows) && installer_file.ends_with(".tar.gz") {
        let output = Command::new("tar")
            .args(&["-xzf", archive_path.to_str().unwrap(), "-C", deps_dir.to_str().unwrap()])
            .output()?;
        if !output.status.success() {
            return Err(format!("Extraction failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        let _ = fs::remove_file(&archive_path);
    }

    Ok(())
}

fn download_file(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(windows) {
        let download_cmd = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; $ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri '{}' -OutFile '{}'",
            url, dest.display()
        );
        let output = Command::new("powershell").args(&["-Command", &download_cmd]).output()?;
        if !output.status.success() {
            return Err(format!("PowerShell download failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
    } else {
        let output = Command::new("curl")
            .args(&["-L", url, "-o", dest.to_str().unwrap()])
            .output()?;
        if !output.status.success() {
            return Err(format!("curl download failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
    }
    Ok(())
}
