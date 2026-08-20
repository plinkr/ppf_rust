# ppf_rust

[English](README.md) | **Español**

Suite de alto rendimiento en Rust para crear, aplicar, inspeccionar y revertir parches en formato **PlayStation Patch File (PPF v1.0, v2.0 y v3.0)**.

El proyecto está organizado como un **Cargo Workspace** modular compuesto por una biblioteca central (`ppf-core`), una herramienta de línea de comandos (`ppf-cli`), una interfaz gráfica de escritorio (`ppf-gui`) desarrollada con `egui`, un paquete WebAssembly (`ppf-wasm`) y una versión web con frontend estático (`web/`) en HTML5 y JavaScript.

Para las herramientas de escritorio (`ppf-cli` y `ppf-gui`), implementa concurrencia con Rayon y operaciones de lectura/escritura mediante mapeo de memoria (`memmap2`), ofreciendo máxima velocidad de procesamiento con seguridad de memoria estricta.

---

<div align="center">
  <p style="max-width:900px; margin:0 auto;">Capturas de pantalla (haz clic en una miniatura para verla en tamaño completo):</p>
  <div style="margin-top:12px; overflow-x:auto; white-space:nowrap; padding:8px 4px; -webkit-overflow-scrolling:touch;">
    <a href="https://github.com/user-attachments/assets/8d67a416-680f-4f7f-8428-a011c8c96a14" target="_blank" rel="noopener">
      <img src="https://github.com/user-attachments/assets/8d67a416-680f-4f7f-8428-a011c8c96a14" width="280" style="display:inline-block; margin-right:8px; border-radius:8px; box-shadow:0 6px 18px rgba(0,0,0,0.12);" alt="Crear Parche" />
    </a>
    <a href="https://github.com/user-attachments/assets/8cedf8c9-c1e0-4ca8-90fc-6af382b209d6" target="_blank" rel="noopener">
      <img src="https://github.com/user-attachments/assets/8cedf8c9-c1e0-4ca8-90fc-6af382b209d6" width="280" style="display:inline-block; margin-right:8px; border-radius:8px; box-shadow:0 6px 18px rgba(0,0,0,0.12);" alt="Aplicar y Revertir" />
    </a>
    <a href="https://github.com/user-attachments/assets/b026ae1b-615e-4f27-b795-a5f65e7ab363" target="_blank" rel="noopener">
      <img src="https://github.com/user-attachments/assets/b026ae1b-615e-4f27-b795-a5f65e7ab363" width="280" style="display:inline-block; margin-right:8px; border-radius:8px; box-shadow:0 6px 18px rgba(0,0,0,0.12);" alt="Información y DIZ" />
    </a>
  </div>
</div>

---

## Versión Web en Línea (GitHub Pages)

Puedes acceder a la versión web de la aplicación directamente desde el navegador en:

**[https://plinkr.github.io/ppf_rust](https://plinkr.github.io/ppf_rust)**

La versión web ejecuta el motor `ppf-core` compilado a **WebAssembly (WASM)** y cuenta con un frontend en **HTML5, CSS y JavaScript**. El procesamiento se realiza al 100% de forma local en tu navegador mediante Web Workers, por lo que ningún archivo ni imagen de disco se sube a internet.

> **Nota importante sobre el rendimiento:**
> La versión WebAssembly opera sobre búferes en memoria (`Uint8Array`) y, por restricciones inherentes al entorno del navegador, **no puede utilizar mapeo de memoria (`memmap2`) ni paralelismo multinúcleo (`rayon`)**.
> 
> Como consecuencia, su velocidad de procesamiento es considerablemente inferior a las versiones nativas y el consumo de RAM en el navegador aumenta con imágenes de gran tamaño (como archivos BIN/ISO de 500 MB o más). **Se aconseja usar la versión web en última instancia o por conveniencia rápida cuando no tengas acceso a los binarios de escritorio. Para un rendimiento óptimo en archivos grandes, utiliza siempre la versión CLI (`ppf_cli`) o la versión GUI (`ppf_gui`).**

---

## Estructura del Workspace

```text
ppf_rust/
├── Cargo.toml                  # Configuración raíz del workspace
├── crates/
│   ├── ppf-core/               # Biblioteca del motor PPF (API pública modular con feature flags)
│   ├── ppf-cli/                # Interfaz de línea de comandos (binario `ppf_cli`)
│   ├── ppf-gui/                # Interfaz gráfica de escritorio en egui (binario `ppf_gui`)
│   └── ppf-wasm/               # Bindings WebAssembly (wasm-bindgen)
└── web/                        # Cliente web estático (HTML5, JavaScript, Web Workers, CSS)
```

- **`ppf-core`**: Motor central desacoplado de la interfaz. Provee parsing, validación de integridad (Blockcheck), creación paralela, aplicación y reversión de parches con callbacks de progreso. Utiliza *feature flags* (`mmap`, `parallel`, `serde`) para habilitar dependencias nativas en escritorio o compilar de forma ligera para WebAssembly.
- **`ppf-cli`**: Aplicación de terminal (`ppf_cli`) basada en `clap` e `indicatif`, diseñada para máxima velocidad, automatización y scripts en consola.
- **`ppf-gui`**: Aplicación de escritorio (`ppf_gui`) multiplataforma basada en `eframe` / `egui`, con soporte para arrastrar y soltar (Drag & Drop), modo seguro con copia de respaldo e inspección visual de parches.
- **`ppf-wasm`**: Capa de integración WebAssembly que exporta las funciones del motor (`apply_patch`, `undo_patch`, `create_patch`, `inspect_patch`) hacia JavaScript mediante `wasm-bindgen`.
- **`web`**: Frontend web estático con interfaz de usuario en pestañas, soporte de Drag & Drop y procesamiento asíncrono en segundo plano mediante Web Workers.

---

## Características

- **Compatibilidad con estándares PPF**: Soporte completo para lectura y aplicación de versiones **PPF 1.0**, **PPF 2.0** y **PPF 3.0**.
- **Creación optimizada de PPF3**: Escaneo de diferencias en paralelo por bloques mediante `rayon` en escritorio, aprovechando todos los núcleos de CPU disponibles.
- **Validación de integridad (Blockcheck)**: Verificación del bloque de validación de 1024 bytes para imágenes BIN estándar (offset `0x9320`) y formato GI/PrimoDVD (offset `0x80A0`).
- **Soporte para datos de reversión (Undo Data)**: Generación y aplicación de parches reversibles para restaurar binarios modificados a su estado original bit a bit.
- **Soporte de metadatos FILE_ID.DIZ**: Inserción y extracción de descripciones extendidas bajo el estándar Amiga/BBS (hasta 3072 bytes).
- **Interfaz Gráfica (GUI) en egui**: Aplicación visual moderna con soporte de Drag & Drop, selección entre parcheo directo (*in-place*) o copia segura (*safe copy*), visores de metadatos y modales de ayuda.
- **Interfaz de Línea de Comandos (CLI)**: Subcomandos estructurados (`apply`, `undo`, `info`, `create`) con barras de progreso interactivas y cálculo de tiempo estimado (ETA).
- **Versión Web (WebAssembly)**: Aplicación web cliente sin dependencias de servidor, lista para usar desde el navegador en GitHub Pages.
- **Motor desacoplado y condicional**: `ppf-core` gestiona mediante *features* el uso de `memmap2` y `rayon`, manteniendo el máximo rendimiento en escritorio sin comprometer la portabilidad hacia WebAssembly.

---

## Compilación e Instalación

### Requisitos previos

- **Rust**: Versión 1.85 o superior (Rust Edition 2024).
- **Cargo**.
- *(Opcional para WebAssembly)*: Target `wasm32-unknown-unknown` y `wasm-bindgen-cli`.

---

### 1. Compilar todo el proyecto (Workspace completo)

Para compilar todos los crates de escritorio del workspace (`ppf-core`, `ppf-cli` y `ppf-gui`) en modo optimizado:

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

### 5. Compilar la versión WebAssembly (`ppf-wasm`) y preparar el cliente web

Si deseas compilar los bindings WebAssembly y generar los paquetes necesarios para servir el frontend web de forma local:

```bash
# 1. Instalar el target WebAssembly
rustup target add wasm32-unknown-unknown

# 2. Instalar la herramienta wasm-bindgen-cli
cargo install wasm-bindgen-cli --version 0.2.100 --locked

# 3. Compilar el crate ppf-wasm para WebAssembly
cargo build --package ppf-wasm --target wasm32-unknown-unknown --release

# 4. Generar los enlaces JavaScript en el directorio web/pkg
wasm-bindgen --target web --out-dir web/pkg --out-name ppf_wasm target/wasm32-unknown-unknown/release/ppf_wasm.wasm
```

Para probar el cliente web localmente, sirve el directorio `web/` con cualquier servidor estático HTTP:

```bash
# Con Python 3:
python3 -m http.server 8080 -d web

# O con herramientas como `basic-http-server`:
basic-http-server web

# O `miniserve`:
miniserve --index index.html --interfaces 127.0.0.1 --port 8080 web/
```

Y abre en tu navegador la dirección `http://localhost:8080`.

---

### 6. Instalación global en el sistema

Puedes instalar los ejecutables nativos directamente en tu directorio `~/.cargo/bin`:

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

## Uso de la Versión Web

La versión web está disponible en línea en **[https://plinkr.github.io/ppf_rust](https://plinkr.github.io/ppf_rust)** y ofrece las mismas capacidades funcionales a través del navegador:

1. **Pestaña Aplicar / Revertir (Apply & Undo)**:
   - Carga la imagen binaria (`.bin`, `.iso`, `.img`, `.cue`, `.raw`) y el parche `.ppf` (vía selector o arrastrando con Drag & Drop).
   - Inspección en vivo de los metadatos y validación de la imagen.
   - Aplica o revierte el parche en memoria y descarga automáticamente el archivo binario resultante.

2. **Pestaña Crear (Create Patch)**:
   - Permite cargar el archivo original y el modificado.
   - Opciones configurables: descripción, tipo de imagen (BIN/GI), validación de bloque (Blockcheck), datos de reversión (Undo) y archivo `FILE_ID.DIZ`.
   - Generación y descarga del parche `.ppf` procesado en un Web Worker en segundo plano.

3. **Pestaña Inspeccionar (Info & DIZ)**:
   - Muestra detalles completos de la cabecera PPF e incluye un visor de texto para descripciones `FILE_ID.DIZ` con botón de copia al portapapeles.

4. **Guía de Ayuda (Help & Guide)**:
   - Instrucciones paso a paso sobre el formato PPF y soluciones a incidencias habituales.

> *Recordatorio de rendimiento:* La versión web es una alternativa práctica cuando no es posible instalar binarios de escritorio. No obstante, al carecer de `rayon` y `memmap2`, para imágenes de disco pesadas se aconseja priorizar las aplicaciones de escritorio (`ppf_cli` o `ppf_gui`).

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
# Por defecto incluye soporte para memoria mapeada (memmap2) y escaneo paralelo (rayon)
ppf-core = { git = "https://github.com/plinkr/ppf_rust.git" }
```

O si utilizas una ruta local en tu propio workspace:

```toml
[dependencies]
ppf-core = { path = "../ppf_rust/crates/ppf-core" }
```

### Configuración de Features (`ppf-core`)

`ppf-core` permite seleccionar las características necesarias según el entorno de destino:

- **`mmap`** *(por defecto)*: Habilita el soporte de lectura y escritura eficiente en disco mediante `memmap2` (`apply_patch`, `undo_patch`, `create_patch`, `inspect_patch`, `PpfFile`).
- **`parallel`** *(por defecto)*: Habilita el procesamiento multinúcleo con `rayon` para la creación de parches.
- **`serde`** *(opcional)*: Habilita `Serialize` y `Deserialize` para estructuras de datos (`PpfHeader`, `PpfCreatorOptions`).

Para entornos embebidos o compilación para WebAssembly (`wasm32-unknown-unknown`), puedes desactivar las dependencias por defecto para operar exclusivamente con búferes en memoria (`&[u8]` y `&mut [u8]`):

```toml
[dependencies]
ppf-core = { git = "https://github.com/plinkr/ppf_rust.git", default-features = false, features = ["serde"] }
```

---

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

### Ejemplo 2: Aplicar o revertir un parche en disco con callbacks de progreso

```rust
use ppf_core::{apply_patch, undo_patch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let on_start = |total_records: usize| {
        println!("Iniciando procesamiento de {} registros...", total_records);
    };

    let on_progress = |records_done: usize| {
        // Invocado por cada bloque o registro aplicado
    };

    // Aplicar parche directamente al binario en disco
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

### Ejemplo 4: Operaciones en memoria o WebAssembly (`_slice`)

Si operas en entornos WebAssembly o procesas búferes en memoria sin tocar el disco:

```rust
use ppf_core::{apply_patch_slice, inspect_patch_slice, undo_patch_slice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let patch_bytes = std::fs::read("parche.ppf")?;
    let mut target_bytes = std::fs::read("juego.bin")?;

    // Inspeccionar cabecera desde bytes
    let info = inspect_patch_slice(&patch_bytes)?;
    println!("Parche para imagen: {:?}", info.image_type);

    // Aplicar modificaciones directamente al slice mutable
    apply_patch_slice(&patch_bytes, &mut target_bytes, None, None)?;

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

### 4. Aplicación de parche con modificaciones aleatorias - 50 ejecuciones (Warm Cache / En memoria)

Prueba de estrés realizada aplicando un parche compuesto por un volumen masivo de modificaciones distribuidas de manera completamente aleatoria en el binario ppf.

En este escenario con dispersión aleatoria extrema, la diferencia de rendimiento entre ambas implementaciones se vuelve muy notable, la versión de referencia en C (`applyppf3`) demora más de 14 segundos debido a la sobrecarga del esquema clásico de I/O, mientras que la implementación en Rust (`ppf_cli`) mediante memoria mapeada (`memmap2`) completa la operación de forma consistente en menos de 900 ms (~826 ms), resultando más de **17 veces más rápida**.

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

| Comando | Media | Mínimo | Máximo | Rendimiento Relativo |
|:---|---:|---:|---:|---:|
| `ppf_cli apply` (Rust) | **826.0 ms ± 11.7 ms** | 788.9 ms | 853.4 ms | **17.08x más rápido** |
| `applyppf3` (C) | 14.108 s ± 0.335 s | 13.711 s | 15.820 s | Referencia |

---

## Licencia

Este proyecto está distribuido bajo los términos de la Licencia MIT. Consulta el archivo [LICENSE](LICENSE) para más detalles.
