# Typesafe LaTeX Editor

Typesafe is a fast, modern LaTeX editor built with Rust. It features live PDF preview, SyncTeX support, integrated dictionary/spellchecking, and extensive autocompletions.

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
- `tectonic.exe` and `pdfium.dll` (in `deps/`)
- `latex_data.json` (in project root)
- `dictionary.txt` (in project root)
- `icon.png` (in `deps/`)

## Development (Optional)

### Regenerating Autocomplete Data
The project comes with a pre-generated `latex_data.json`. If you wish to regenerate the autocomplete dataset from the latest TeXStudio CWL files, you can run the ingestion tool:

```bash
cargo run --release --bin ingest_cwl
```

### Dependencies
The `deps/` folder contains the necessary binaries for Windows (`tectonic.exe`, `pdfium.dll`). Ensure these are present if you are cloning the repo freshly (Note: Large binaries might need to be downloaded or tracked via LFS if not in the standard repo).

## Features
*   **Intelligent Autocomplete**: Thousands of LaTeX commands and environments.
*   **Live Preview**: PDF rendering powered by Pdfium.
*   **Inverse Search**: Double-click the PDF to jump to the corresponding line in the editor.
*   **Theming**: Multiple built-in editor themes (Serendipity, Tokyo Night, etc.).
*   **Spellcheck**: Real-time spellchecking.

## License
Refer to the `LICENSE` file for details.