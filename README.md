# Typesafe

A modern LaTeX editor built with Rust and egui.

## Build Instructions

### Windows

The build process is automated to download necessary dependencies (Tectonic and PDFium).

1. **Build & Package**:
   Run the included batch script from the project root:
   ```cmd
   build_package.bat
   ```

2. **Output**:
   This will compile the application and place `typesafe.exe` in the project root, alongside the required `pdfium.dll` and `tectonic.exe`.

### Linux / macOS

On Unix systems, you must provide the dependencies via your system package manager, as the automatic downloader supports Windows only.

1. **Install Dependencies**:
   - **Tectonic**: Install via your package manager (e.g., `apt install tectonic`, `brew install tectonic`).
   - **PDFium**: Install `libpdfium` development libraries.

2. **Build**:
   ```bash
   cargo build --release
   ```

3. **Output**:
   The executable will be located at `target/release/typesafe`.

## Usage

Run the executable to start the editor.

**Windows Note**: The application requires `tectonic.exe` and `pdfium.dll` to be present in the same directory (or the `deps/` subdirectory) to compile documents and render previews.

## License

MIT