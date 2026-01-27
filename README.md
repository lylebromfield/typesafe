# Typesafe LaTeX Editor

Typesafe is a fast, modern LaTeX editor built with Rust. It features live PDF preview, SyncTeX support, integrated dictionary/spellchecking, and extensive autocompletions.

## Requirements

**TeX Live** is required to compile LaTeX documents. Typesafe will automatically prompt you to download and install it on first compile if it's not already installed on your system.

## Building from Source

Typesafe is designed to be easy to build. The application logic automatically locates required runtime resources (dependencies, dictionaries, autocomplete data) in the project root if they are not found immediately next to the executable.

### 1. Build
```bash
cargo build --release
```

### 2. Run
```bash
target/release/typesafe.exe
```

That's it! The application will automatically locate:
- `install-tl-windows.exe` or `install-tl-unx.tar.gz` (TeX Live installer in `deps/`)
- `pdfium.dll` (in `deps/`)
- `latex_data.json` (in project root)
- `dictionary.txt` (in project root)
- `icon.png` (in `deps/`)

On first compile, if TeX Live is not detected on your system, you will be prompted to download and install it.

## Development (Optional)

### Regenerating Autocomplete Data
The project comes with a pre-generated `latex_data.json`. If you wish to regenerate the autocomplete dataset from the latest TeXStudio CWL files, you can run the ingestion tool:

```bash
cargo run --release --bin ingest_cwl
```

### Dependencies
The `deps/` folder contains necessary files for Windows and Unix systems:
- `pdfium.dll` (Windows PDF rendering)
- `install-tl-windows.exe` (Windows TeX Live installer)
- `install-tl-unx.tar.gz` (Unix TeX Live installer)

Ensure these are present if you are cloning the repo freshly.

## Features
*   **Full BibLaTeX Support**: Native support for APA 7th citations and other BibLaTeX styles through TeX Live.
*   **Intelligent Autocomplete**: Thousands of LaTeX commands and environments.
*   **Live Preview**: PDF rendering powered by Pdfium.
*   **Inverse Search**: Double-click the PDF to jump to the corresponding line in the editor.
*   **Pop-out PDF Viewer**: Open PDF preview in a separate window with Ctrl+Shift+P.
*   **Markdown Support**: Preview Markdown files alongside LaTeX editing.
*   **Theming**: Multiple built-in editor themes (Serendipity, Tokyo Night, etc.).
*   **Spellcheck**: Real-time spellchecking with dictionary support.
*   **PDF File Support**: Open and view PDF files directly in the editor.

## License
Refer to the `LICENSE` file for details.