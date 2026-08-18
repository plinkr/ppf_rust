# ppf_rust

[English](README.md) | **Español**

Suite de alto rendimiento en Rust para crear, aplicar, inspeccionar y revertir parches en formato **PlayStation Patch File (PPF v1.0, v2.0 y v3.0)**.

El proyecto está organizado como un **Cargo Workspace** modular compuesto por una biblioteca central (`ppf-core`), una herramienta de línea de comandos (`ppf-cli`) y una interfaz gráfica de escritorio (`ppf-gui`) desarrollada con `egui`.

Implementa concurrencia con Rayon y lectura/escritura mediante mapeo de memoria (`memmap2`), ofreciendo máxima velocidad de procesamiento con seguridad de memoria estricta.

---

## Estructura del Workspace

```text
ppf_rust/
├── Cargo.toml                  # Configuración raíz del workspace
└── crates/
    ├── ppf-core/               # Biblioteca del motor PPF (API pública)
    ├── ppf-cli/                # Interfaz de línea de comandos (binario `ppf_cli`)
    └── ppf-gui/                # Interfaz gráfica en egui (binario `ppf_gui`)
```

- **`ppf-core`**: Motor central desacoplado de la interfaz. Provee parsing, validación de integridad (Blockcheck), creación paralela, aplicación y reversión de parches con callbacks de progreso.
- **`ppf-cli`**: Aplicación de terminal (`ppf_cli`) basada en `clap` e `indicatif`, diseñada para automatización, scripts y flujos de trabajo en consola.
- **`ppf-gui`**: Aplicación de escritorio (`ppf_gui`) multiplataforma basada en `eframe` / `egui`, con soporte para arrastrar y soltar (Drag & Drop), modo seguro con copia de respaldo e inspección visual de parches.

---

## Características

- **Compatibilidad con estándares PPF**: Soporte completo para lectura y aplicación de versiones **PPF 1.0**, **PPF 2.0** y **PPF 3.0**.
- **Creación optimizada de PPF3**: Escaneo de diferencias en paralelo por bloques mediante `rayon`, aprovechando todos los núcleos de CPU disponibles.
- **Validación de integridad (Blockcheck)**: Verificación del bloque de validación de 1024 bytes para imágenes BIN estándar (offset `0x9320`) y formato GI/PrimoDVD (offset `0x80A0`).
- **Soporte para datos de reversión (Undo Data)**: Generación y aplicación de parches reversibles para restaurar binarios modificados a su estado original bit a bit.
- **Soporte de metadatos FILE_ID.DIZ**: Inserción y extracción de descripciones extendidas bajo el estándar Amiga/BBS (hasta 3072 bytes).
- **Interfaz Gráfica (GUI) en egui**: Aplicación visual moderna con soporte de Drag & Drop, selección entre parcheo directo (*in-place*) o copia segura (*safe copy*), visores de metadatos y modales de ayuda.
- **Interfaz de Línea de Comandos (CLI)**: Subcomandos estructurados (`apply`, `undo`, `info`, `create`) con barras de progreso interactivas y cálculo de tiempo estimado (ETA).
- **Motor desacoplado**: Biblioteca reutilizable con tipos de error tipados (`thiserror`) y callbacks desacoplados para integraciones personalizadas.

---

## Compilación e Instalación

### Requisitos previos

- **Rust**: Versión 1.85 o superior (Rust Edition 2024).
- **Cargo**.

---

### 1. Compilar todo el proyecto (Workspace completo)

Para compilar todos los crates del workspace (`ppf-core`, `ppf-cli` y `ppf-gui`) en modo optimizado:

```bash
cargo build --release
```

o de forma explícita:

```bash
cargo build --workspace --release
```

Los binarios generados se ubicarán en `target/release/`:

- `target/release/ppf_cli` (Herramienta de consola)
- `target/release/ppf_gui` (Interfaz gráfica)

---

### 2. Compilar únicamente el CLI (`ppf-cli`)

Si solo necesitas la herramienta de terminal:

```bash
cargo build -p ppf-cli --release
# o especificando el nombre del binario:
cargo build --bin ppf_cli --release
```

El binario resultante se encontrará en `target/release/ppf_cli`.

---

### 3. Compilar únicamente el GUI (`ppf-gui`)

Si solo deseas compilar la interfaz gráfica de escritorio:

```bash
cargo build -p ppf-gui --release
# o especificando el nombre del binario:
cargo build --bin ppf_gui --release
```

El binario resultante se encontrará en `target/release/ppf_gui`.

---

### 4. Compilar únicamente la biblioteca (`ppf-core`)

Para verificar o compilar la biblioteca central de forma aislada:

```bash
cargo build -p ppf-core --release
```

---

### 5. Instalación global en el sistema

Puedes instalar los ejecutables directamente en tu directorio `~/.cargo/bin`:

```bash
# Instalar el CLI globalmente
cargo install --path crates/ppf-cli

# Instalar el GUI globalmente
cargo install --path crates/ppf-gui
```

---

## Ejecución

### Ejecución directa con Cargo

```bash
# Ejecutar la interfaz gráfica (GUI)
cargo run -p ppf-gui --release

# Ejecutar el CLI
cargo run -p ppf-cli -- --help
cargo run -p ppf-cli -- info --patch parche.ppf
```

---

## Uso de la Interfaz Gráfica (`ppf_gui`)

Ejecuta el binario `ppf_gui`:

```bash
./target/release/ppf_gui
```

### Funcionalidades de la GUI:

1. **Pestaña Aplicar / Revertir (Apply & Undo)**:

   - **Selección de archivos**: Carga la imagen binaria (`.bin`, `.iso`, `.img`, `.cue`, `.raw`) y el parche `.ppf` mediante los selectores o arrastrando los archivos directamente a la ventana (**Drag & Drop**).
   - **Modo de operación**:
     - *Patch in-place*: Aplica o revierte modificaciones directamente en el archivo original (rápido, sin duplicar espacio en disco).
     - *Safe Copy*: Genera automáticamente una copia modificada (ej. `juego_patched.bin`), manteniendo el binario original intacto.
   - **Inspección automática**: Al seleccionar un parche, se analiza e informa en tiempo real su versión (PPF1, PPF2, PPF3), descripción, presencia de Undo data y estado de Blockcheck.
   - **Progreso en vivo**: Barra de progreso con porcentaje durante operaciones pesadas.

2. **Pestaña Crear (Create Patch)**:

   - Permite seleccionar el archivo original y el archivo modificado.
   - Opciones configurables: descripción del parche (hasta 50 caracteres), tipo de imagen (BIN estándar o GI), verificación de bloque (Blockcheck), inclusión de datos de reversión (Undo data) y archivo opcional `FILE_ID.DIZ`.
   - Ejecución asíncrona en segundo plano sin bloquear la interfaz gráfica.

3. **Pestaña Inspeccionar (Info & DIZ)**:

   - Inspección detallada de cabeceras de parches PPF (versión, tamaño, flags, integridad).
   - Visor integrado de texto para metadatos `FILE_ID.DIZ` con botón de copia rápida al portapapeles.

4. **Guía de Ayuda (Help & Guide)**:

   - Modal de ayuda integrado con explicaciones detalladas para principiantes sobre cómo aplicar, revertir y crear parches, junto con resolución de problemas comunes.

---

## Uso de la Línea de Comandos (`ppf_cli`)

El binario `ppf_cli` ofrece comandos directos para terminal y scripts:

```text
Uso: ppf_cli <COMANDO>

Comandos:
  apply   Aplica un parche PPF a un archivo binario
  undo    Revierte un parche PPF3 de un archivo binario
  info    Muestra la información y metadatos de un parche PPF
  create  Crea un nuevo parche PPF3 comparando dos archivos binarios
  help    Muestra la ayuda de los comandos disponibles
```

### 1. Inspeccionar un parche (`info`)

Muestra la versión del parche, descripción, tipo de imagen, estado del blockcheck y contenido de `FILE_ID.DIZ` si existe.

```bash
ppf_cli info --patch juego.ppf
```

### 2. Aplicar un parche (`apply`)

Aplica las modificaciones del parche sobre el binario destino de forma directa (in-place).

```bash
ppf_cli apply --bin juego.bin --patch traduccion.ppf
```

### 3. Revertir un parche (`undo`)

Restaura el binario a su estado original previo a la aplicación del parche (requiere que el parche haya sido creado con datos de Undo).

```bash
ppf_cli undo --bin juego.bin --patch traduccion.ppf
```

### 4. Crear un parche (`create`)

Compara el binario original y el modificado para generar un archivo PPF3 optimizado:

```bash
ppf_cli create \
  --original juego_original.bin \
  --patched juego_modificado.bin \
  --output parche.ppf \
  --description "Traducción al Español v1.0" \
  --undo \
  --file-id descripcion.diz
```

#### Opciones de `create`:

- `-O, --original <PATH>`: Archivo binario original sin modificar.
- `-p, --patched <PATH>`: Archivo binario con las modificaciones aplicadas.
- `-o, --output <PATH>`: Ruta de destino donde se guardará el archivo `.ppf`.
- `-u, --undo`: Incluye datos de restauración para permitir revertir el parche con `undo`.
- `-x, --disable-validation`: Desactiva la verificación de integridad de bloque (Blockcheck).
- `-i, --imagetype <0|1>`: Tipo de imagen (`0` = BIN/RAW estándar [por defecto], `1` = GI / PrimoDVD).
- `-d, --description <TEXTO>`: Descripción de hasta 50 caracteres incrustada en la cabecera.
- `-f, --file-id <PATH>`: Archivo de texto opcional que se incrustará como metadatos `FILE_ID.DIZ` (hasta 3072 bytes).

---

## Uso como Biblioteca (`ppf-core`)

Para integrar el motor de parcheo en tu propio proyecto de Rust, añade `ppf-core` a tu `Cargo.toml`:

```toml
[dependencies]
ppf-core = { git = "https://github.com/plinkr/ppf_rust.git" }
```

O si utilizas una ruta local en tu propio workspace:

```toml
[dependencies]
ppf-core = { path = "../ppf_rust/crates/ppf-core" }
```

### Ejemplo 1: Inspeccionar metadatos de un parche

```rust
use ppf_core::inspect_patch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let info = inspect_patch("patch.ppf")?;

    println!("Versión PPF  : {:?}", info.version);
    println!("Descripción  : {}", info.description);
    println!("Tiene Undo   : {}", info.has_undo);
    println!("Block Check  : {}", info.block_check);

    if let Some(diz) = info.file_id {
        println!("FILE_ID.DIZ:\n{}", diz);
    }

    Ok(())
}
```

### Ejemplo 2: Aplicar o revertir un parche con callbacks de progreso

La biblioteca permite conectar callbacks para monitorizar el progreso en interfaces gráficas o consolas personalizadas:

```rust
use ppf_core::{apply_patch, undo_patch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let on_start = |total_records: usize| {
        println!("Iniciando procesamiento de {} registros...", total_records);
    };

    let on_progress = |records_done: usize| {
        // Invocado por cada bloque o registro aplicado
    };

    // Aplicar parche directamente al binario
    apply_patch(
        "traduccion.ppf",
        "juego.bin",
        Some(&on_start),
        Some(&on_progress),
    )?;

    // Revertir parche (si contiene datos de Undo)
    undo_patch(
        "traduccion.ppf",
        "juego.bin",
        Some(&on_start),
        Some(&on_progress),
    )?;

    Ok(())
}
```

### Ejemplo 3: Crear un parche PPF3 con opciones avanzadas

```rust
use ppf_core::{create_patch, ImageType, PpfCreatorOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = PpfCreatorOptions {
        description: "Mi Parche v1.0".to_string(),
        image_type: ImageType::Bin,
        block_check: true,
        undo_data: true,
        file_id: Some(b"Metadatos extendidos del parche".to_vec()),
    };

    let progress_callback = |bytes_scanned: usize| {
        // Invocado conforme se procesan bloques en paralelo
    };

    let total_diffs = create_patch(
        "original.bin",
        "modificado.bin",
        "salida.ppf",
        &options,
        Some(&progress_callback),
    )?;

    println!("Parche generado exitosamente con {} diferencias.", total_diffs);
    Ok(())
}
```

---

## Pruebas

Para ejecutar la suite completa de pruebas unitarias y de integración en todo el workspace:

```bash
cargo test --workspace
```

O si tienes instalado `cargo-nextest`:

```bash
cargo nextest run --release --workspace
```

---

## Rendimiento y Benchmarks

Comparativa de rendimiento realizada con `hyperfine` (50 ejecuciones) frente a la implementación de referencia en C ([meunierd/ppf](https://github.com/meunierd/ppf) - `makeppf3` y `applyppf3`), utilizando una imagen de disco de *Castlevania: Symphony of the Night* (514 MB).

### 1. Creación de parche - 50 ejecuciones (Cold Cache / Sin caché en RAM)

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

| Comando | Media | Mínimo | Máximo | Rendimiento Relativo |
|:---|---:|---:|---:|---:|
| `ppf_cli create` (Rust) | **3.981 s ± 0.072 s** | 3.825 s | 4.199 s | **1.28x más rápido** |
| `makeppf3` (C) | 5.096 s ± 0.174 s | 4.828 s | 5.735 s | Referencia |

### 2. Creación de parche - 50 ejecuciones (Warm Cache / En memoria)

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

| Comando | Media | Mínimo | Máximo | Rendimiento Relativo |
|:---|---:|---:|---:|---:|
| `ppf_cli create` (Rust) | **88.2 ms ± 2.5 ms** | 85.8 ms | 101.5 ms | **7.61x más rápido** |
| `makeppf3` (C) | 670.8 ms ± 13.8 ms | 650.2 ms | 725.6 ms | Referencia |

### 3. Aplicación de parche - 50 ejecuciones (Warm Cache / En memoria)

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

| Comando | Media | Mínimo | Máximo | Rendimiento Relativo |
|:---|---:|---:|---:|---:|
| `ppf_cli apply` (Rust) | **2.9 ms ± 0.1 ms** | 2.8 ms | 3.4 ms | **46.11x más rápido** |
| `applyppf3` (C) | 135.3 ms ± 4.9 ms | 127.8 ms | 155.4 ms | Referencia |


---

## Licencia

Este proyecto está distribuido bajo los términos de la Licencia MIT. Consulta el archivo [LICENSE](LICENSE) para más detalles.
