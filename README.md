# NeoLatex

**English** | [Português](README.pt-BR.md)

A fast, elegant LaTeX editor written in Rust. Your source on one side, the compiled PDF on the other, updated as you type.

![NeoLatex](docs/screenshot.png)

## Features

- Editor with LaTeX syntax highlighting, line numbers and current-line highlight
- Side-by-side PDF preview with zoom (`Ctrl` + scroll), fit-to-width and a page indicator
- Background compilation: the UI never blocks and no console window pops up
- Automatic recompilation shortly after you stop typing
- Compilation log with colored errors and warnings
- Drag a `.tex` file onto the window to open it

## Requirements

- **Windows** (fonts and console suppression rely on Windows-specific features)
- [Rust](https://rustup.rs) (recent stable toolchain)
- A LaTeX distribution with `pdflatex` on your `PATH`, such as [MiKTeX](https://miktex.org) or [TeX Live](https://tug.org/texlive/)
- `pdfium.dll` (x64), used to render the PDF

## Installation

```bash
git clone https://github.com/giseldo/neolatex.git
cd neolatex
```

Download `pdfium.dll` from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries/releases) (`pdfium-win-x64.tgz`, inside `bin/`) and place it in the project folder and next to the executable. From PowerShell:

```powershell
curl.exe -L -o pdfium.tgz https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-win-x64.tgz
mkdir pdfium-tmp; tar -xzf pdfium.tgz -C pdfium-tmp
Copy-Item pdfium-tmp\bin\pdfium.dll .
Remove-Item -Recurse -Force pdfium-tmp, pdfium.tgz
```

## Usage

```bash
cargo run --release
```

The executable is built to `target/release/neolatex.exe`. Copy `pdfium.dll` into that folder if you run it directly.

Without `pdfium.dll` the editor still works, but the PDF pane shows an error message.

## Shortcuts

| Shortcut | Action |
| --- | --- |
| `F5` | Compile |
| `Ctrl+S` | Save and compile |
| `Ctrl+O` / `Ctrl+N` | Open / new |
| `Ctrl` + `=` / `-` / `0` | Increase / decrease / reset editor font size |
| `Ctrl` + scroll over the PDF | Zoom the preview |

## Built with

[egui](https://github.com/emilk/egui) / eframe for the UI, [pdfium-render](https://github.com/ajrcarey/pdfium-render) to render the PDF, and `pdflatex` to compile.

## Known limitations

- The document is compiled in a temporary folder, so paths relative to your `.tex` file (`\input`, `\includegraphics`) are not resolved yet.
- PDF pages are rendered on the UI thread, so very long documents may stall briefly on each compile.

## License

[MIT](LICENSE)
