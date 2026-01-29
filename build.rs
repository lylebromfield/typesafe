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
    let pdfium_file = if cfg!(windows) {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    };

    let pdfium_path = deps_dir.join(pdfium_file);
    if !pdfium_path.exists() {
        println!("cargo:warning=Downloading {}...", pdfium_file);
        if let Err(e) = download_pdfium(&deps_dir, pdfium_file) {
            println!("cargo:warning=Failed to download pdfium: {}", e);
        }
    }

    let tectonic_file = if cfg!(windows) { "tectonic.exe" } else { "tectonic" };
    let tectonic_path = deps_dir.join(tectonic_file);
    if !tectonic_path.exists() {
        println!("cargo:warning=Downloading Tectonic...");
        if let Err(e) = download_tectonic(&deps_dir, tectonic_file) {
            println!("cargo:warning=Failed to download Tectonic: {}", e);
        }
    }

    let biber_file = if cfg!(windows) { "biber.exe" } else { "biber" };
    let biber_path = deps_dir.join(biber_file);
    if !biber_path.exists() {
        println!("cargo:warning=Downloading Biber...");
        if let Err(e) = download_biber(&deps_dir, biber_file) {
            println!("cargo:warning=Failed to download Biber: {}", e);
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

fn download_tectonic(deps_dir: &Path, _target_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let version = "0.15.0";
    let url = if cfg!(windows) {
        format!("https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic@{0}/tectonic-{0}-x86_64-pc-windows-msvc.zip", version)
    } else if cfg!(target_os = "macos") {
        format!("https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic@{0}/tectonic-{0}-x86_64-apple-darwin.tar.gz", version)
    } else {
        format!("https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic@{0}/tectonic-{0}-x86_64-unknown-linux-gnu.tar.gz", version)
    };

    let archive_name = if cfg!(windows) { "tectonic.zip" } else { "tectonic.tar.gz" };
    let archive_path = deps_dir.join(archive_name);

    download_file(&url, &archive_path)?;

    // Extract
    if cfg!(windows) {
        let unzip_cmd = format!(
            "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
            archive_path.display(), deps_dir.display()
        );
        let output = Command::new("powershell").args(&["-Command", &unzip_cmd]).output()?;
        if !output.status.success() {
            return Err(format!("Unzip failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
    } else {
        let output = Command::new("tar")
            .args(&["-xzf", archive_path.to_str().unwrap(), "-C", deps_dir.to_str().unwrap()])
            .output()?;
        if !output.status.success() {
            return Err(format!("Extraction failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
    }

    let _ = fs::remove_file(&archive_path);
    Ok(())
}

fn download_biber(deps_dir: &Path, _target_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let version = "2.17";
    let url = if cfg!(windows) {
        format!("https://downloads.sourceforge.net/project/biblatex-biber/biblatex-biber/{}/binaries/Windows/biber-MSWIN64.zip", version)
    } else if cfg!(target_os = "macos") {
        format!("https://downloads.sourceforge.net/project/biblatex-biber/biblatex-biber/{}/binaries/OSX_Intel/biber-darwin_x86_64.tar.gz", version)
    } else {
        format!("https://downloads.sourceforge.net/project/biblatex-biber/biblatex-biber/{}/binaries/Linux/biber-linux_x86_64.tar.gz", version)
    };

    let archive_name = if cfg!(windows) { "biber.zip" } else { "biber.tar.gz" };
    let archive_path = deps_dir.join(archive_name);

    download_file(&url, &archive_path)?;

    // Extract
    if cfg!(windows) {
        let unzip_cmd = format!(
            "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
            archive_path.display(), deps_dir.display()
        );
        let output = Command::new("powershell").args(&["-Command", &unzip_cmd]).output()?;
        if !output.status.success() {
            return Err(format!("Unzip failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
    } else {
        let output = Command::new("tar")
            .args(&["-xzf", archive_path.to_str().unwrap(), "-C", deps_dir.to_str().unwrap()])
            .output()?;
        if !output.status.success() {
            return Err(format!("Extraction failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
    }

    let _ = fs::remove_file(&archive_path);
    Ok(())
}

fn download_file(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(windows) {
        let download_cmd = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; $ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri '{}' -OutFile '{}' -UserAgent 'TypesafeBuildScript/1.0'",
            url, dest.display()
        );
        let output = Command::new("powershell").args(&["-Command", &download_cmd]).output()?;
        if !output.status.success() {
            return Err(format!("PowerShell download failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
    } else {
        let output = Command::new("curl")
            .args(&["-L", "-A", "TypesafeBuildScript/1.0", url, "-o", dest.to_str().unwrap()])
            .output()?;
        if !output.status.success() {
            return Err(format!("curl download failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
    }
    Ok(())
}
