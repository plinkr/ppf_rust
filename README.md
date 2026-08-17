# ppf_rust

**English** | [Español](README_ES.md)

A high-performance Rust library and CLI tool for creating, applying, inspecting, and reverting patches in the **PlayStation Patch File (PPF v1.0, v2.0, and v3.0)** format.

Implements concurrency with Rayon and memory-mapped I/O (`memmap2`), delivering efficient performance with memory safety. The patching engine logic is completely decoupled from the user interface, enabling integration into terminal applications as well as future graphical interfaces.

---

## Features

- **PPF Standards Compatibility**: Full support for reading and applying **PPF 1.0**, **PPF 2.0**, and **PPF 3.0** versions.
- **Optimized PPF3 Creation**: Block-level parallel difference scanning using `rayon`, leveraging multiple CPU cores.
- **Integrity Validation (Blockcheck)**: 1024-byte block validation for standard BIN images (offset `0x9320`) and GI/PrimoDVD format (offset `0x80A0`).
- **Undo Data Support**: Generation and application of reversible patches to restore modified binaries bit-by-bit to their original state.
- **FILE_ID.DIZ Metadata Support**: Insertion and extraction of extended descriptions under the Amiga/BBS standard (up to 3072 bytes).
- **Decoupled Library**: UI-agnostic engine with progress callbacks (`on_start`, `progress_callback`) and typed error handling via `thiserror`.

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
  --command-name 'Rust Version (ppf_rust) - cold' \
  --prepare 'rm -f /tmp/out_rust.ppf; sync; echo 3 | sudo tee /proc/sys/vm/drop_caches > /dev/null' \
  'ppf_rust/target/release/ppf_rust create --original ppf_rust/target/CastlevaniaSOTN-orig.bin --patched ppf_rust/target/CastlevaniaSOTN-patched.bin --output /tmp/out_rust.ppf --description "bench"'
Benchmark 1: C version (makeppf3) - cold
  Time (mean ± σ):      5.096 s ±  0.174 s    [User: 0.869 s, System: 0.833 s]
  Range (min … max):    4.828 s …  5.735 s    50 runs

Benchmark 2: Rust Version (ppf_rust) - cold
  Time (mean ± σ):      3.981 s ±  0.072 s    [User: 0.204 s, System: 1.182 s]
  Range (min … max):    3.825 s …  4.199 s    50 runs

Summary
  Rust Version (ppf_rust) - cold ran
    1.28 ± 0.05 times faster than C Version (makeppf3) - cold
```

| Command | Mean | Min | Max | Relative Performance |
|:---|---:|---:|---:|---:|
| `ppf_rust create` (Rust) | **3.981 s ± 0.072 s** | 3.825 s | 4.199 s | **1.28x faster** |
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
  --command-name 'Rust Version (ppf_rust) - warm' \
  --prepare 'rm -f /tmp/out_rust.ppf' \
  'ppf_rust/target/release/ppf_rust create --original ppf_rust/target/CastlevaniaSOTN-orig.bin --patched ppf_rust/target/CastlevaniaSOTN-patched.bin --output /tmp/out_rust.ppf --description "bench"' \
  --style full
Benchmark 1: C Version (makeppf3) - warm
[sudo] password for plinkr:
  Time (mean ± σ):     670.8 ms ±  13.8 ms    [User: 526.1 ms, System: 141.6 ms]
  Range (min … max):   650.2 ms … 725.6 ms    50 runs

Benchmark 2: Rust Version (ppf_rust) - warm
  Time (mean ± σ):      88.2 ms ±   2.5 ms    [User: 256.2 ms, System: 141.0 ms]
  Range (min … max):    85.8 ms … 101.5 ms    50 runs

Summary
  Rust Version (ppf_rust) - warm ran
    7.61 ± 0.27 times faster than C Version (makeppf3) - warm
```

| Command | Mean | Min | Max | Relative Performance |
|:---|---:|---:|---:|---:|
| `ppf_rust create` (Rust) | **88.2 ms ± 2.5 ms** | 85.8 ms | 101.5 ms | **7.61x faster** |
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
  --command-name 'Rust Version (ppf_rust) - warm' \
  --prepare 'cp ppf_rust/target/CastlevaniaSOTN-orig.bin /tmp/apply_rust.bin' \
  'ppf_rust/target/release/ppf_rust apply --bin /tmp/apply_rust.bin --patch /tmp/out_rust.ppf'
Benchmark 1: C Version (applyppf3) - warm
  Time (mean ± σ):     135.3 ms ±   4.9 ms    [User: 72.9 ms, System: 62.0 ms]
  Range (min … max):   127.8 ms … 155.4 ms    50 runs

Benchmark 2: Rust Version (ppf_rust) - warm
  Time (mean ± σ):       2.9 ms ±   0.1 ms    [User: 1.9 ms, System: 1.2 ms]
  Range (min … max):     2.8 ms …   3.4 ms    50 runs

  Warning: Command took less than 5 ms to complete. Note that the results might be inaccurate because hyperfine can not calibrate the shell startup time much more precise than this limit. You can try to use the `-N`/`--shell=none` option to disable the shell completely.

Summary
  Rust Version (ppf_rust) - warm ran
   46.11 ± 2.40 times faster than C Version (applyppf3) - warm
```

| Command | Mean | Min | Max | Relative Performance |
|:---|---:|---:|---:|---:|
| `ppf_rust apply` (Rust) | **2.9 ms ± 0.1 ms** | 2.8 ms | 3.4 ms | **46.11x faster** |
| `applyppf3` (C) | 135.3 ms ± 4.9 ms | 127.8 ms | 155.4 ms | Reference |

---

## Installation & Build

### Requirements
- **Rust**: 1.85 or later (Rust Edition 2024).
- **Cargo**.

### Building from source
```bash
git clone https://github.com/plinkr/ppf_rust.git
cd ppf_rust
cargo build --release
```

The compiled binary executable will be available at `target/release/ppf_cli`.

To install the CLI globally on your system:
```bash
cargo install --path .
```

---

## CLI Usage (`ppf_cli`)

The `ppf_cli` binary provides a structured command-line interface organized with subcommands.

```text
Usage: ppf_cli <COMMAND>

Commands:
  apply   Apply a PPF patch to a binary file
  undo    Revert a PPF3 patch from a binary file
  info    Display information and metadata of a PPF patch
  create  Create a new PPF3 patch by comparing two binary files
```

### 1. Inspect a patch (`info`)
Displays the patch version, description, image type, blockcheck status, and `FILE_ID.DIZ` contents if present.

```bash
ppf_cli info --patch game.ppf
```

### 2. Apply a patch (`apply`)
Applies patch modifications directly (in-place) to the target binary.

```bash
ppf_cli apply --bin game.bin --patch translation.ppf
```

### 3. Undo a patch (`undo`)
Restores the binary to its original state prior to patch application (requires the patch to have been created with undo data).

```bash
ppf_cli undo --bin game.bin --patch translation.ppf
```

### 4. Create a patch (`create`)
Compares the original and modified binaries to generate an optimized PPF3 file.

```bash
ppf_cli create \
  --original original_game.bin \
  --patched modified_game.bin \
  --output patch.ppf \
  --description "English Translation v1.0" \
  --undo \
  --file-id description.diz
```

#### `create` options:
- `-O, --original <PATH>`: Unmodified original binary file.
- `-p, --patched <PATH>`: Binary file with modifications applied.
- `-o, --output <PATH>`: Output path where the `.ppf` patch will be written.
- `-u, --undo`: Include restore data (allows using the `undo` subcommand).
- `-x, --disable-validation`: Disable block integrity verification (Blockcheck).
- `-i, --imagetype <0|1>`: Image type (`0` = standard BIN/RAW [default], `1` = GI / PrimoDVD).
- `-d, --description <TEXT>`: Description of up to 50 characters embedded in the header.
- `-f, --file-id <PATH>`: Optional text file embedded as `FILE_ID.DIZ` metadata (up to 3072 bytes).

---

## Library Usage (`ppf_rust`)

Add the library to your `Cargo.toml` dependencies:

```toml
[dependencies]
ppf_rust = { path = "../ppf_rust" } # or from git `ppf_rust = { git = "https://github.com/plinkr/ppf_rust.git" }`
```

### Example 1: Inspect patch metadata

```rust
use ppf_rust::inspect_patch;

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

### Example 2: Apply or revert a patch with progress callbacks

The library allows attaching callbacks to monitor progress in graphical interfaces or custom consoles.

```rust
use ppf_rust::{apply_patch, undo_patch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let on_start = |total_records: usize| {
        println!("Starting application of {} records...", total_records);
    };

    let on_progress = |records_done: usize| {
        // Invoked for each applied record
    };

    // Apply patch
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

### Example 3: Create a PPF3 patch in memory or to disk

```rust
use ppf_rust::{create_patch, ImageType, PpfCreatorOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = PpfCreatorOptions {
        description: "My Patch v1.0".to_string(),
        image_type: ImageType::Bin,
        block_check: true,
        undo_data: true,
        file_id: Some(b"Extended patch metadata".to_vec()),
    };

    let progress_callback = |bytes_scanned: usize| {
        // Invoked for each block processed in parallel
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

## Roadmap

- [x] High-performance PPF 1.0, 2.0, and 3.0 patching engine.
- [x] Command-line interface (`ppf_cli`) with interactive progress bars.
- [x] Concurrent patch generation with Rayon and memory mapping.
- [ ] **Graphical User Interface (GUI)**: Desktop visual interface implementation.

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
