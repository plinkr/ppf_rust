# ppf_rust

**English** | [Español](README_ES.md)

High-performance Rust suite for creating, applying, inspecting, and reverting patches in the **PlayStation Patch File (PPF v1.0, v2.0, and v3.0)** format.

The project is organized as a modular **Cargo Workspace** comprising a core engine library (`ppf-core`), a command-line interface tool (`ppf-cli`), and a desktop graphical user interface (`ppf-gui`) built with `egui`.

Implements concurrency with Rayon and memory-mapped I/O (`memmap2`), delivering maximum processing throughput with strict memory safety.

---

## Workspace Structure

```text
ppf_rust/
├── Cargo.toml                  # Root workspace configuration
└── crates/
    ├── ppf-core/               # PPF engine library (public API)
    ├── ppf-cli/                # Command-line interface (`ppf_cli` binary)
    └── ppf-gui/                # Graphical interface in egui (`ppf_gui` binary)
```

- **`ppf-core`**: Core engine decoupled from the UI. Provides parsing, block validation (Blockcheck), parallel diff generation, patch application, and patch undo with progress callbacks.
- **`ppf-cli`**: Terminal application (`ppf_cli`) powered by `clap` and `indicatif`, built for automation, scripting, and command-line workflows.
- **`ppf-gui`**: Cross-platform desktop application (`ppf_gui`) built with `eframe` / `egui`, featuring Drag & Drop support, safe backup copy mode, and visual patch inspection.

---

## Features

- **PPF Standards Compatibility**: Full support for reading and applying **PPF 1.0**, **PPF 2.0**, and **PPF 3.0** versions.
- **Optimized PPF3 Creation**: Block-level parallel difference scanning using `rayon`, using all available CPU cores.
- **Integrity Validation (Blockcheck)**: 1024-byte validation block verification for standard BIN images (offset `0x9320`) and GI/PrimoDVD format (offset `0x80A0`).
- **Undo Data Support**: Generation and application of reversible patches to restore modified binaries bit-by-bit to their original state.
- **FILE_ID.DIZ Metadata Support**: Insertion and extraction of extended descriptions under the Amiga/BBS standard (up to 3072 bytes).
- **Graphical User Interface (GUI) in egui**: Modern visual application featuring Drag & Drop, toggle between in-place patching and safe backup copies, metadata inspector, FILE_ID.DIZ viewer with clipboard copy, and an integrated help guide.
- **Command-Line Interface (CLI)**: Structured subcommands (`apply`, `undo`, `info`, `create`) with interactive progress bars and ETA calculation.
- **Decoupled Engine**: Reusable library with typed error handling (`thiserror`) and progress callbacks for custom integrations.

---

## Compilation & Installation

### Prerequisites

- **Rust**: Version 1.85 or later (Rust Edition 2024).
- **Cargo**.

---

### 1. Build the Entire Workspace

To compile all workspace crates (`ppf-core`, `ppf-cli`, and `ppf-gui`) in release mode:

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

### 5. System-wide Installation

You can install the executables directly to your `~/.cargo/bin` directory:

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

## Graphical User Interface (`ppf_gui`)

Run the `ppf_gui` binary:

```bash
./target/release/ppf_gui
```

### GUI Features:

1. **Apply & Undo Tab**:

   - **File Selection**: Load the target disc image (`.bin`, `.iso`, `.img`, `.cue`, `.raw`) and the `.ppf` patch using the file choosers or by dragging and dropping them directly onto the window (**Drag & Drop**).
   - **Operation Mode**:
     - *Patch in-place*: Directly modifies the original binary file (fast, requires no extra disk space).
     - *Safe Copy*: Automatically creates a modified copy (e.g. `game_patched.bin`), preserving the original file untouched.
   - **Automatic Inspection**: Inspects the patch upon selection and displays its version (PPF1, PPF2, PPF3), description, Undo data availability, and Blockcheck status in real time.
   - **Live Progress**: Progress bar with percentage display during operations.

2. **Create Patch Tab**:

   - Select original unmodified binary and modified binary files.
   - Configurable options: patch description (up to 50 characters), image type (standard BIN or GI), block validation (Blockcheck), reversible undo data generation, and optional `FILE_ID.DIZ` file.
   - Non-blocking asynchronous background execution keeping the UI responsive.

3. **Info & DIZ Tab**:

   - Detailed inspection of PPF patch headers (version, image type, validation flags, undo availability).
   - Integrated text viewer for `FILE_ID.DIZ` release metadata with one-click copy to clipboard.

4. **Help & Guide**:

   - Integrated help modal explaining step-by-step how to apply, undo, and create patches, along with common troubleshooting tips.

---

## Command-Line Interface (`ppf_cli`)

The `ppf_cli` binary provides direct commands for terminal usage and scripting:

```text
Usage: ppf_cli <COMMAND>

Commands:
  apply   Apply a PPF patch to a binary file
  undo    Undo a PPF3 patch from a binary file
  info    Show patch information
  create  Create a new PPF patch from two binary files
  help    Print this message or the help of the given subcommand(s)
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
- `-o, --output <PATH>`: Output path where the `.ppf` file will be saved.
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
ppf-core = { git = "https://github.com/plinkr/ppf_rust.git" }
```

Or using a local path within your workspace:

```toml
[dependencies]
ppf-core = { path = "../ppf_rust/crates/ppf-core" }
```

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

### Example 2: Apply or Revert a Patch with Progress Callbacks

The library allows attaching callbacks to monitor progress in GUI or custom console applications:

```rust
use ppf_core::{apply_patch, undo_patch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let on_start = |total_records: usize| {
        println!("Starting processing of {} records...", total_records);
    };

    let on_progress = |records_done: usize| {
        // Invoked for each applied record/block
    };

    // Apply patch directly to binary
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

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
