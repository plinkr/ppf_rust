# ppf_rust

**English** | [Español](README_ES.md)

High-performance Rust suite for creating, applying, inspecting, and reverting patches in the **PlayStation Patch File (PPF v1.0, v2.0, and v3.0)** format.

The project is organized as a modular **Cargo Workspace** comprising a core engine library (`ppf-core`), a command-line interface tool (`ppf-cli`), a desktop graphical user interface (`ppf-gui`) built with `egui`, a WebAssembly package (`ppf-wasm`), and a static web client (`web/`) in HTML5 and JavaScript.

For desktop tools (`ppf-cli` and `ppf-gui`), it implements concurrency with Rayon and memory-mapped I/O (`memmap2`), delivering maximum processing throughput with strict memory safety.

---

<div align="center">
  <p style="max-width:900px; margin:0 auto;">Screenshots (click a thumbnail to view full size):</p>
  <div style="margin-top:12px; overflow-x:auto; white-space:nowrap; padding:8px 4px; -webkit-overflow-scrolling:touch;">
    <a href="https://github.com/user-attachments/assets/8d67a416-680f-4f7f-8428-a011c8c96a14" target="_blank" rel="noopener">
      <img src="https://github.com/user-attachments/assets/8d67a416-680f-4f7f-8428-a011c8c96a14" width="280" style="display:inline-block; margin-right:8px; border-radius:8px; box-shadow:0 6px 18px rgba(0,0,0,0.12);" alt="Create Patch" />
    </a>
    <a href="https://github.com/user-attachments/assets/8cedf8c9-c1e0-4ca8-90fc-6af382b209d6" target="_blank" rel="noopener">
      <img src="https://github.com/user-attachments/assets/8cedf8c9-c1e0-4ca8-90fc-6af382b209d6" width="280" style="display:inline-block; margin-right:8px; border-radius:8px; box-shadow:0 6px 18px rgba(0,0,0,0.12);" alt="Apply & Undo" />
    </a>
    <a href="https://github.com/user-attachments/assets/b026ae1b-615e-4f27-b795-a5f65e7ab363" target="_blank" rel="noopener">
      <img src="https://github.com/user-attachments/assets/b026ae1b-615e-4f27-b795-a5f65e7ab363" width="280" style="display:inline-block; margin-right:8px; border-radius:8px; box-shadow:0 6px 18px rgba(0,0,0,0.12);" alt="Info & DIZ" />
    </a>
  </div>
</div>

---

## Online Web Version (GitHub Pages)

You can access the web version of the application directly in your browser at:

**[https://plinkr.github.io/ppf_rust](https://plinkr.github.io/ppf_rust)**

The web version runs the `ppf-core` engine compiled to **WebAssembly (WASM)** and features a frontend built with **HTML5, CSS, and JavaScript**. Processing is 100% local in your browser using Web Workers, so no files or disc images are uploaded to the internet.

> **Important performance note:**
> The WebAssembly version operates on in-memory buffers (`Uint8Array`) and, due to inherent browser environment constraints, **cannot use memory mapping (`memmap2`) or multi-core parallelism (`rayon`)**.
> 
> As a result, its processing speed is considerably slower than the native versions and browser RAM consumption increases with large images (such as BIN/ISO files of 500 MB or more). **Using the web version is recommended as a last resort or for quick convenience when desktop binaries are unavailable. For optimal performance on large files, always use the CLI version (`ppf_cli`) or the GUI version (`ppf_gui`).**

---

## Workspace Structure

```text
ppf_rust/
├── Cargo.toml                  # Root workspace configuration
├── crates/
│   ├── ppf-core/               # PPF engine library (modular public API with feature flags)
│   ├── ppf-cli/                # Command-line interface (`ppf_cli` binary)
│   ├── ppf-gui/                # Desktop graphical interface in egui (`ppf_gui` binary)
│   └── ppf-wasm/               # WebAssembly bindings (wasm-bindgen)
└── web/                        # Static web client (HTML5, JavaScript, Web Workers, CSS)
```

- **`ppf-core`**: Core engine decoupled from the UI. Provides parsing, integrity validation (Blockcheck), parallel diff generation, patch application, and patch undo with progress callbacks. Uses *feature flags* (`mmap`, `parallel`, `serde`) to enable native dependencies on desktop or compile lightweight builds for WebAssembly.
- **`ppf-cli`**: Terminal application (`ppf_cli`) powered by `clap` and `indicatif`, built for maximum speed, automation, and command-line scripting.
- **`ppf-gui`**: Cross-platform desktop application (`ppf_gui`) built with `eframe` / `egui`, featuring Drag & Drop support, safe backup copy mode, and visual patch inspection.
- **`ppf-wasm`**: WebAssembly integration layer exporting engine functions (`apply_patch`, `undo_patch`, `create_patch`, `inspect_patch`) to JavaScript via `wasm-bindgen`.
- **`web`**: Static web frontend with tabbed UI, Drag & Drop support, and asynchronous background processing via Web Workers.

---

## Features

- **PPF Standards Compatibility**: Full support for reading and applying **PPF 1.0**, **PPF 2.0**, and **PPF 3.0** versions.
- **Optimized PPF3 Creation**: Block-level parallel difference scanning using `rayon` on desktop, utilizing all available CPU cores.
- **Integrity Validation (Blockcheck)**: 1024-byte validation block verification for standard BIN images (offset `0x9320`) and GI/PrimoDVD format (offset `0x80A0`).
- **Undo Data Support**: Generation and application of reversible patches to restore modified binaries bit-by-bit to their original state.
- **FILE_ID.DIZ Metadata Support**: Insertion and extraction of extended descriptions under the Amiga/BBS standard (up to 3072 bytes).
- **Graphical User Interface (GUI) in egui**: Modern visual application featuring Drag & Drop, toggle between direct (*in-place*) patching and safe copy (*safe copy*), metadata viewers, and help modals.
- **Command-Line Interface (CLI)**: Structured subcommands (`apply`, `undo`, `info`, `create`) with interactive progress bars and ETA calculation.
- **Web Version (WebAssembly)**: Client-side web application with zero server dependencies, ready to use in the browser via GitHub Pages.
- **Decoupled and Conditional Engine**: `ppf-core` manages `memmap2` and `rayon` via *features*, maintaining maximum desktop performance without compromising WebAssembly portability.

---

## Compilation & Installation

### Prerequisites

- **Rust**: Version 1.85 or later (Rust Edition 2024).
- **Cargo**.
- *(Optional for WebAssembly)*: `wasm32-unknown-unknown` target and `wasm-bindgen-cli`.

---

### 1. Build the Entire Workspace

To compile all workspace desktop crates (`ppf-core`, `ppf-cli`, and `ppf-gui`) in release mode:

```bash
cargo build --release
```

or explicitly:

```bash
cargo build --workspace --release
```

The compiled binaries will be placed in `target/release/`:

- `target/release/ppf_cli` (CLI tool)
- `target/release/ppf_gui` (Graphical interface)

---

### 2. Build the CLI Only (`ppf-cli`)

If you only need the terminal tool:

```bash
cargo build -p ppf-cli --release
# or specifying the binary name:
cargo build --bin ppf_cli --release
```

The resulting binary will be located at `target/release/ppf_cli`.

---

### 3. Build the GUI Only (`ppf-gui`)

If you only want to build the desktop graphical interface:

```bash
cargo build -p ppf-gui --release
# or specifying the binary name:
cargo build --bin ppf_gui --release
```

The resulting binary will be located at `target/release/ppf_gui`.

---

### 4. Build the Library Only (`ppf-core`)

To check or build the core engine library in isolation:

```bash
cargo build -p ppf-core --release
```

---

### 5. Build the WebAssembly Version (`ppf-wasm`) and Prepare the Web Client

If you want to build the WebAssembly bindings and generate the packages needed to serve the web frontend locally:

```bash
# 1. Install WebAssembly target
rustup target add wasm32-unknown-unknown

# 2. Install wasm-bindgen-cli tool
cargo install wasm-bindgen-cli --version 0.2.100 --locked

# 3. Build ppf-wasm crate for WebAssembly
cargo build --package ppf-wasm --target wasm32-unknown-unknown --release

# 4. Generate JavaScript bindings in web/pkg directory
wasm-bindgen --target web --out-dir web/pkg --out-name ppf_wasm target/wasm32-unknown-unknown/release/ppf_wasm.wasm
```

To test the web client locally, serve the `web/` directory with any static HTTP server:

```bash
# With Python 3:
python3 -m http.server 8080 -d web

# Or with tools like `basic-http-server`:
basic-http-server web

# Or `miniserve`:
miniserve --index index.html --interfaces 127.0.0.1 --port 8080 web/
```

And open `http://localhost:8080` in your browser.

---

### 6. System-wide Installation

You can install the native executables directly to your `~/.cargo/bin` directory:

```bash
# Install the CLI globally
cargo install --path crates/ppf-cli

# Install the GUI globally
cargo install --path crates/ppf-gui
```

---

## Running

### Direct Execution with Cargo

```bash
# Launch the graphical user interface (GUI)
cargo run -p ppf-gui --release

# Run the CLI
cargo run -p ppf-cli -- --help
cargo run -p ppf-cli -- info --patch patch.ppf
```

---

## Web Version Usage

The web version is available online at **[https://plinkr.github.io/ppf_rust](https://plinkr.github.io/ppf_rust)** and offers the same functional capabilities in your browser:

1. **Apply / Undo Tab (Apply & Undo)**:
   - Load the target binary image (`.bin`, `.iso`, `.img`, `.cue`, `.raw`) and the `.ppf` patch (via file picker or Drag & Drop).
   - Live inspection of metadata and image validation.
   - Applies or reverts the patch in memory and automatically downloads the resulting binary file.

2. **Create Patch Tab**:
   - Allows loading original and modified files.
   - Configurable options: description, image type (BIN/GI), block validation (Blockcheck), undo data, and `FILE_ID.DIZ` file.
   - Generates and downloads the `.ppf` patch processed asynchronously in a background Web Worker.

3. **Inspect Tab (Info & DIZ)**:
   - Displays full PPF header details and includes a text viewer for `FILE_ID.DIZ` descriptions with a copy-to-clipboard button.

4. **Help Guide (Help & Guide)**:
   - Step-by-step instructions on the PPF format and solutions to common issues.

> *Performance reminder:* The web version is a practical alternative when desktop binaries cannot be installed. However, lacking `rayon` and `memmap2`, desktop applications (`ppf_cli` or `ppf_gui`) should be prioritized for heavy disc images.

---

## Graphical User Interface (`ppf_gui`)

Run the `ppf_gui` binary:

```bash
./target/release/ppf_gui
```

### GUI Features:

1. **Apply / Undo Tab (Apply & Undo)**:

   - **File Selection**: Load the target binary image (`.bin`, `.iso`, `.img`, `.cue`, `.raw`) and the `.ppf` patch using file pickers or by dragging and dropping files directly onto the window (**Drag & Drop**).
   - **Operation Mode**:
     - *Patch in-place*: Directly modifies the original binary file (fast, requires no extra disk space).
     - *Safe Copy*: Automatically creates a modified copy (e.g. `game_patched.bin`), keeping the original binary untouched.
   - **Automatic Inspection**: Upon selecting a patch, it analyzes and reports in real time its version (PPF1, PPF2, PPF3), description, Undo data presence, and Blockcheck status.
   - **Live Progress**: Progress bar with percentage during heavy operations.

2. **Create Patch Tab**:

   - Allows selecting the original unmodified file and the modified file.
   - Configurable options: patch description (up to 50 characters), image type (standard BIN or GI), block validation (Blockcheck), reversible undo data inclusion, and optional `FILE_ID.DIZ` file.
   - Asynchronous background execution without blocking the graphical interface.

3. **Inspect Tab (Info & DIZ)**:

   - Detailed inspection of PPF patch headers (version, size, flags, integrity).
   - Integrated text viewer for `FILE_ID.DIZ` metadata with a quick copy-to-clipboard button.

4. **Help Guide (Help & Guide)**:

   - Integrated help modal with detailed explanations for beginners on how to apply, revert, and create patches, along with common troubleshooting steps.

---

## Command-Line Interface (`ppf_cli`)

The `ppf_cli` binary provides direct commands for terminal usage and scripting:

```text
Usage: ppf_cli <COMMAND>

Commands:
  apply   Apply a PPF patch to a binary file
  undo    Undo a PPF3 patch from a binary file
  info    Show patch information and metadata
  create  Create a new PPF3 patch comparing two binary files
  help    Print help for available commands
```

### 1. Inspect a Patch (`info`)

Displays the patch version, description, image type, blockcheck status, and `FILE_ID.DIZ` contents if present.

```bash
ppf_cli info --patch game.ppf
```

### 2. Apply a Patch (`apply`)

Applies patch modifications directly to the target binary file (in-place).

```bash
ppf_cli apply --bin game.bin --patch translation.ppf
```

### 3. Revert a Patch (`undo`)

Restores the binary file to its original state prior to patch application (requires the patch to have been created with Undo data).

```bash
ppf_cli undo --bin game.bin --patch translation.ppf
```

### 4. Create a Patch (`create`)

Compares the original and modified binaries to generate an optimized PPF3 patch:

```bash
ppf_cli create \
  --original original_game.bin \
  --patched modified_game.bin \
  --output patch.ppf \
  --description "English Translation v1.0" \
  --undo \
  --file-id description.diz
```

#### `create` Options:

- `-O, --original <PATH>`: Unmodified original binary file.
- `-p, --patched <PATH>`: Binary file with modifications applied.
- `-o, --output <PATH>`: Destination path where the `.ppf` file will be saved.
- `-u, --undo`: Include restore data to allow reverting the patch with `undo`.
- `-x, --disable-validation`: Disable block integrity verification (Blockcheck).
- `-i, --imagetype <0|1>`: Image type (`0` = standard BIN/RAW [default], `1` = GI / PrimoDVD).
- `-d, --description <TEXT>`: Description up to 50 characters embedded in the header.
- `-f, --file-id <PATH>`: Optional text file embedded as `FILE_ID.DIZ` metadata (up to 3072 bytes).

---

## Library Usage (`ppf-core`)

To integrate the patching engine into your own Rust project, add `ppf-core` to your `Cargo.toml`:

```toml
[dependencies]
# By default includes memory-mapped file support (memmap2) and parallel scanning (rayon)
ppf-core = { git = "https://github.com/plinkr/ppf_rust.git" }
```

Or using a local path within your workspace:

```toml
[dependencies]
ppf-core = { path = "../ppf_rust/crates/ppf-core" }
```

### Feature Flags Configuration (`ppf-core`)

`ppf-core` allows selecting the required features depending on your target environment:

- **`mmap`** *(default)*: Enables efficient disk read and write support via `memmap2` (`apply_patch`, `undo_patch`, `create_patch`, `inspect_patch`, `PpfFile`).
- **`parallel`** *(default)*: Enables multi-core processing with `rayon` for patch creation.
- **`serde`** *(optional)*: Enables `Serialize` and `Deserialize` on data structures (`PpfHeader`, `PpfCreatorOptions`).

For embedded environments or WebAssembly targets (`wasm32-unknown-unknown`), you can disable default dependencies to operate exclusively on in-memory buffers (`&[u8]` and `&mut [u8]`):

```toml
[dependencies]
ppf-core = { git = "https://github.com/plinkr/ppf_rust.git", default-features = false, features = ["serde"] }
```

---

### Example 1: Inspect Patch Metadata

```rust
use ppf_core::inspect_patch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let info = inspect_patch("patch.ppf")?;

    println!("PPF Version  : {:?}", info.version);
    println!("Description  : {}", info.description);
    println!("Has Undo     : {}", info.has_undo);
    println!("Block Check  : {}", info.block_check);

    if let Some(diz) = info.file_id {
        println!("FILE_ID.DIZ:\n{}", diz);
    }

    Ok(())
}
```

### Example 2: Apply or Revert a Patch on Disk with Progress Callbacks

```rust
use ppf_core::{apply_patch, undo_patch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let on_start = |total_records: usize| {
        println!("Starting processing of {} records...", total_records);
    };

    let on_progress = |records_done: usize| {
        // Invoked for each applied record or block
    };

    // Apply patch directly to binary on disk
    apply_patch(
        "translation.ppf",
        "game.bin",
        Some(&on_start),
        Some(&on_progress),
    )?;

    // Revert patch (if it contains Undo data)
    undo_patch(
        "translation.ppf",
        "game.bin",
        Some(&on_start),
        Some(&on_progress),
    )?;

    Ok(())
}
```

### Example 3: Create a PPF3 Patch with Advanced Options

```rust
use ppf_core::{create_patch, ImageType, PpfCreatorOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = PpfCreatorOptions {
        description: "My Patch v1.0".to_string(),
        image_type: ImageType::Bin,
        block_check: true,
        undo_data: true,
        file_id: Some(b"Extended patch metadata".to_vec()),
    };

    let progress_callback = |bytes_scanned: usize| {
        // Invoked as blocks are processed in parallel
    };

    let total_diffs = create_patch(
        "original.bin",
        "modified.bin",
        "output.ppf",
        &options,
        Some(&progress_callback),
    )?;

    println!("Patch generated successfully with {} differences.", total_diffs);
    Ok(())
}
```

### Example 4: In-Memory or WebAssembly Operations (`_slice`)

If operating in WebAssembly environments or processing in-memory buffers without touching disk:

```rust
use ppf_core::{apply_patch_slice, inspect_patch_slice, undo_patch_slice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let patch_bytes = std::fs::read("patch.ppf")?;
    let mut target_bytes = std::fs::read("game.bin")?;

    // Inspect header from bytes
    let info = inspect_patch_slice(&patch_bytes)?;
    println!("Patch for image type: {:?}", info.image_type);

    // Apply modifications directly to the mutable slice
    apply_patch_slice(&patch_bytes, &mut target_bytes, None, None)?;

    Ok(())
}
```

---

## Testing

To run the full test suite across the entire workspace:

```bash
cargo test --workspace
```

Or using `cargo-nextest`:

```bash
cargo nextest run --release --workspace
```

---

## Performance & Benchmarks

Performance comparison conducted with `hyperfine` (50 runs) against the reference C implementation ([meunierd/ppf](https://github.com/meunierd/ppf) - `makeppf3` and `applyppf3`), using a disc image of *Castlevania: Symphony of the Night* (514 MB).

### 1. Patch Creation - 50 runs (Cold Cache / No RAM cache)

```bash
hyperfine \
  --runs 50 \
  --export-markdown makeppf_cold_results.md \
  --command-name 'C Version (makeppf3) - cold' \
  --prepare 'rm -f /tmp/out_c.ppf; sync; echo 3 | sudo tee /proc/sys/vm/drop_caches > /dev/null' \
  'ppf-master/ppfdev/makeppf_src/makeppf3 c -d "bench" ppf_rust/target/CastlevaniaSOTN-orig.bin ppf_rust/target/CastlevaniaSOTN-patched.bin /tmp/out_c.ppf' \
  --command-name 'Rust Version (ppf_cli) - cold' \
  --prepare 'rm -f /tmp/out_rust.ppf; sync; echo 3 | sudo tee /proc/sys/vm/drop_caches > /dev/null' \
  'ppf_rust/target/release/ppf_cli create --original ppf_rust/target/CastlevaniaSOTN-orig.bin --patched ppf_rust/target/CastlevaniaSOTN-patched.bin --output /tmp/out_rust.ppf --description "bench"'
Benchmark 1: C version (makeppf3) - cold
  Time (mean ± σ):      5.096 s ±  0.174 s    [User: 0.869 s, System: 0.833 s]
  Range (min … max):    4.828 s …  5.735 s    50 runs

Benchmark 2: Rust Version (ppf_cli) - cold
  Time (mean ± σ):      3.981 s ±  0.072 s    [User: 0.204 s, System: 1.182 s]
  Range (min … max):    3.825 s …  4.199 s    50 runs

Summary
  Rust Version (ppf_cli) - cold ran
    1.28 ± 0.05 times faster than C Version (makeppf3) - cold
```

| Command | Mean | Min | Max | Relative Performance |
|:---|---:|---:|---:|---:|
| `ppf_cli create` (Rust) | **3.981 s ± 0.072 s** | 3.825 s | 4.199 s | **1.28x faster** |
| `makeppf3` (C) | 5.096 s ± 0.174 s | 4.828 s | 5.735 s | Reference |

### 2. Patch Creation - 50 runs (Warm Cache / In-Memory)

```bash
hyperfine \
  --warmup 3 \
  --runs 50 \
  --export-markdown makeppf_warm_results.md \
  --setup 'sync && echo 3 | sudo tee /proc/sys/vm/drop_caches > /dev/null' \
  --command-name 'C Version (makeppf3) - warm' \
  --prepare 'rm -f /tmp/out_c.ppf' \
  'ppf-master/ppfdev/makeppf_src/makeppf3 c -d "bench" ppf_rust/target/CastlevaniaSOTN-orig.bin ppf_rust/target/CastlevaniaSOTN-patched.bin /tmp/out_c.ppf' \
  --command-name 'Rust Version (ppf_cli) - warm' \
  --prepare 'rm -f /tmp/out_rust.ppf' \
  'ppf_rust/target/release/ppf_cli create --original ppf_rust/target/CastlevaniaSOTN-orig.bin --patched ppf_rust/target/CastlevaniaSOTN-patched.bin --output /tmp/out_rust.ppf --description "bench"' \
  --style full
Benchmark 1: C Version (makeppf3) - warm
[sudo] password for plinkr:
  Time (mean ± σ):     670.8 ms ±  13.8 ms    [User: 526.1 ms, System: 141.6 ms]
  Range (min … max):   650.2 ms … 725.6 ms    50 runs

Benchmark 2: Rust Version (ppf_cli) - warm
  Time (mean ± σ):      88.2 ms ±   2.5 ms    [User: 256.2 ms, System: 141.0 ms]
  Range (min … max):    85.8 ms … 101.5 ms    50 runs

Summary
  Rust Version (ppf_cli) - warm ran
    7.61 ± 0.27 times faster than C Version (makeppf3) - warm
```

| Command | Mean | Min | Max | Relative Performance |
|:---|---:|---:|---:|---:|
| `ppf_cli create` (Rust) | **88.2 ms ± 2.5 ms** | 85.8 ms | 101.5 ms | **7.61x faster** |
| `makeppf3` (C) | 670.8 ms ± 13.8 ms | 650.2 ms | 725.6 ms | Reference |

### 3. Patch Application - 50 runs (Warm Cache / In-Memory)

```bash
hyperfine \
  --warmup 5 \
  --runs 50 \
  --export-markdown apply_patch_warm_results.md \
  --setup 'sync && echo 3 | sudo tee /proc/sys/vm/drop_caches >/dev/null' \
  --command-name 'C Version (applyppf3) - warm' \
  --prepare 'cp ppf_rust/target/CastlevaniaSOTN-orig.bin /tmp/apply_c.bin' \
  'ppf-master/ppfdev/applyppf_src/applyppf3 a /tmp/apply_c.bin /tmp/out_rust.ppf' \
  --command-name 'Rust Version (ppf_cli) - warm' \
  --prepare 'cp ppf_rust/target/CastlevaniaSOTN-orig.bin /tmp/apply_rust.bin' \
  'ppf_rust/target/release/ppf_cli apply --bin /tmp/apply_rust.bin --patch /tmp/out_rust.ppf'
Benchmark 1: C Version (applyppf3) - warm
  Time (mean ± σ):     135.3 ms ±   4.9 ms    [User: 72.9 ms, System: 62.0 ms]
  Range (min … max):   127.8 ms … 155.4 ms    50 runs

Benchmark 2: Rust Version (ppf_cli) - warm
  Time (mean ± σ):       2.9 ms ±   0.1 ms    [User: 1.9 ms, System: 1.2 ms]
  Range (min … max):     2.8 ms …   3.4 ms    50 runs

Summary
  Rust Version (ppf_cli) - warm ran
   46.11 ± 2.40 times faster than C Version (applyppf3) - warm
```

| Command | Mean | Min | Max | Relative Performance |
|:---|---:|---:|---:|---:|
| `ppf_cli apply` (Rust) | **2.9 ms ± 0.1 ms** | 2.8 ms | 3.4 ms | **46.11x faster** |
| `applyppf3` (C) | 135.3 ms ± 4.9 ms | 127.8 ms | 155.4 ms | Reference |

### 4. Patch Application with Random Modifications - 50 runs (Warm Cache / In-Memory)

Stress test conducted by applying a patch consisting of a massive volume of modifications distributed completely at random throughout the ppf binary.

In this extreme random scatter scenario, the performance difference between both implementations becomes very noticeable, the reference C version (`applyppf3`) takes over 14 seconds due to the system call overhead of traditional I/O, while the Rust implementation (`ppf_cli`) using memory mapping (`memmap2`) consistently finishes in under 900 ms (~826 ms), running over **17 times faster**.

```bash
Benchmark 1: C Version (applyppf3) - warm
  Time (mean ± σ):     14.108 s ±  0.335 s    [User: 7.292 s, System: 6.772 s]
  Range (min … max):   13.711 s … 15.820 s    50 runs

Benchmark 2: Rust Version (ppf_cli) - warm
  Time (mean ± σ):     826.0 ms ±  11.7 ms    [User: 473.3 ms, System: 349.0 ms]
  Range (min … max):   788.9 ms … 853.4 ms    50 runs

Summary
  Rust Version (ppf_cli) - warm ran
   17.08 ± 0.47 times faster than C Version (applyppf3) - warm
```

| Command | Mean | Min | Max | Relative Performance |
|:---|---:|---:|---:|---:|
| `ppf_cli apply` (Rust) | **826.0 ms ± 11.7 ms** | 788.9 ms | 853.4 ms | **17.08x faster** |
| `applyppf3` (C) | 14.108 s ± 0.335 s | 13.711 s | 15.820 s | Reference |

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
