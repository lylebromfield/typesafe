# Typesafe

Typesafe is a modern, offline-first LaTeX editor built with Rust and egui. It features a real-time PDF preview, intelligent autocomplete, and a distraction-free writing environment.

## Features

- **Real-time Preview**: Instant feedback as you type (powered by `pdfium`).
- **Offline-First**: Uses the `tectonic` typesetting engine which manages packages automatically, but the editor itself works fully offline once packages are cached.
- **Intelligent Autocomplete**:
  - `\ref{}` triggers a popup of all `\label{}` definitions in your project.
  - `\cite{}` triggers a popup of bibliography entries from `.bib` files.
- **Spell Check & Thesaurus**:
  - **Spell Check**: Uses a local dictionary (`dictionary.txt`) to highlight typos. (Downloaded automatically on first run, ~4MB).
  - **Thesaurus**: Right-click context menu provides synonyms via the Datamuse API (requires internet connection).
- **Diagnostics**: Clickable error logs that jump directly to the problematic line.

## Building from Source

To build Typesafe, you need to have **Rust** and **Cargo** installed.

1.  **Install Rust**: Visit [rustup.rs](https://rustup.rs/) to install.
2.  **Clone the Repository**:
    ```bash
    git clone https://github.com/yourusername/typesafe.git
    cd typesafe
    ```
3.  **Build**:
    You can use the included batch script (Windows) or standard cargo commands.

    **Option A: Windows Batch Script**
    ```cmd
    build_package.bat
    ```
    This script performs an **incremental build** (preserving build artifacts in `target/` for faster re-compilation), copies `tectonic.exe` and `pdfium.dll` from the `deps/` folder (if present), and places the final `typesafe.exe` in the root directory.

    **Option B: Manual Build**
    ```bash
    cargo build --release
    ```
    The executable will be located in `target/release/typesafe`.

### Runtime Dependencies

Typesafe requires two external binaries to function correctly. These should be placed in the same directory as the executable or in a `deps/` subdirectory:

1.  **Tectonic**: The LaTeX engine.
    - Windows: `tectonic.exe`
    - Linux/macOS: `tectonic` (or installed via package manager)
2.  **PDFium**: The PDF rendering library.
    - Windows: `pdfium.dll`
    - Linux: `libpdfium.so`
    - macOS: `libpdfium.dylib`

*Note: The `build_package.bat` script handles copying these files automatically if they are present in the `deps/` folder.*

## Customization

- **App Icon**: Place a 256x256 PNG file named `icon.png` in the project root to set the window icon.
- **Dictionary**: The app downloads `dictionary.txt` from GitHub on the first launch. You can manually replace this file with any newline-separated word list to change the spell check dictionary.