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
    let (pdfium_file, tectonic_file) = if cfg!(windows) {
        ("pdfium.dll", "tectonic.exe")
    } else if cfg!(target_os = "macos") {
        ("libpdfium.dylib", "tectonic")
    } else {
        ("libpdfium.so", "tectonic")
    };

    let pdfium_path = deps_dir.join(pdfium_file);
    if !pdfium_path.exists() {
        println!("cargo:warning=Downloading {}...", pdfium_file);
        if let Err(e) = download_pdfium(&deps_dir, pdfium_file) {
            println!("cargo:warning=Failed to download pdfium: {}", e);
        }
    }

    let tectonic_path = deps_dir.join(tectonic_file);
    if !tectonic_path.exists() {
        println!("cargo:warning=Downloading {}...", tectonic_file);
        if let Err(e) = download_tectonic(&deps_dir, tectonic_file) {
            println!("cargo:warning=Failed to download tectonic: {}", e);
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
    let (url, is_zip) = if cfg!(windows) {
        (format!("https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic@{}/tectonic-{}-x86_64-pc-windows-msvc.zip", version, version), true)
    } else if cfg!(target_os = "macos") {
        (format!("https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic@{}/tectonic-{}-x86_64-apple-darwin.tar.gz", version, version), false)
    } else {
        (format!("https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic@{}/tectonic-{}-x86_64-unknown-linux-musl.tar.gz", version, version), false)
    };

    let archive_path = deps_dir.join(if is_zip { "tectonic.zip" } else { "tectonic.tar.gz" });
    download_file(&url, &archive_path)?;

    if is_zip {
        #[cfg(windows)]
        {
            let unzip_cmd = format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                archive_path.display(), deps_dir.display()
            );
            Command::new("powershell").args(&["-Command", &unzip_cmd]).output()?;
        }
        #[cfg(not(windows))]
        {
            Command::new("unzip").args(&["-o", archive_path.to_str().unwrap(), "-d", deps_dir.to_str().unwrap()]).output()?;
        }
    } else {
        Command::new("tar").args(&["-xzf", archive_path.to_str().unwrap(), "-C", deps_dir.to_str().unwrap()]).output()?;
    }

    let _ = fs::remove_file(&archive_path);
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
