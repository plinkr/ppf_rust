# ppf_rust

[English](README.md) | **Español**

Biblioteca y herramienta CLI de alto rendimiento en Rust para crear, aplicar, inspeccionar y revertir parches en formato **PlayStation Patch File (PPF v1.0, v2.0 y v3.0)**.

Implementa concurrencia con Rayon y lectura/escritura mediante mapeo de memoria (`memmap2`), ofreciendo un rendimiento eficiente con seguridad de memoria. La lógica del motor de parcheo está completamente desacoplada de la interfaz de usuario, permitiendo su integración tanto en aplicaciones de terminal como en futuras interfaces gráficas.

---

## Características

- **Compatibilidad con estándares PPF**: Soporte completo para lectura y aplicación de versiones **PPF 1.0**, **PPF 2.0** y **PPF 3.0**.
- **Creación optimizada de PPF3**: Escaneo de diferencias en paralelo por bloques mediante `rayon`, aprovechando múltiples núcleos de CPU.
- **Validación de integridad (Blockcheck)**: Validación del bloque de 1024 bytes para imágenes BIN estándar (offset `0x9320`) y formato GI/PrimoDVD (offset `0x80A0`).
- **Soporte para datos de reversión (Undo Data)**: Generación y aplicación de parches reversibles para restaurar binarios modificados a su estado original bit a bit.
- **Soporte de metadatos FILE_ID.DIZ**: Inserción y extracción de descripciones extendidas bajo el estándar Amiga/BBS (hasta 3072 bytes).
- **Biblioteca desacoplada**: Motor agnóstico a la interfaz con callbacks de progreso (`on_start`, `progress_callback`) y manejo tipado de errores con `thiserror`.

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

| Comando | Media | Mínimo | Máximo | Rendimiento Relativo |
|:---|---:|---:|---:|---:|
| `ppf_rust create` (Rust) | **3.981 s ± 0.072 s** | 3.825 s | 4.199 s | **1.28x más rápido** |
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

| Comando | Media | Mínimo | Máximo | Rendimiento Relativo |
|:---|---:|---:|---:|---:|
| `ppf_rust create` (Rust) | **88.2 ms ± 2.5 ms** | 85.8 ms | 101.5 ms | **7.61x más rápido** |
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

| Comando | Media | Mínimo | Máximo | Rendimiento Relativo |
|:---|---:|---:|---:|---:|
| `ppf_rust apply` (Rust) | **2.9 ms ± 0.1 ms** | 2.8 ms | 3.4 ms | **46.11x más rápido** |
| `applyppf3` (C) | 135.3 ms ± 4.9 ms | 127.8 ms | 155.4 ms | Referencia |

---

## Instalación y Compilación

### Requisitos
- **Rust**: 1.85 o superior (Rust Edition 2024).
- **Cargo**.

### Compilación desde el código fuente
```bash
git clone https://github.com/plinkr/ppf_rust.git
cd ppf_rust
cargo build --release
```

El binario ejecutable compilado estará disponible en `target/release/ppf_cli`.

Para instalar el CLI globalmente en el sistema:
```bash
cargo install --path .
```

---

## Uso del CLI (`ppf_cli`)

El binario `ppf_cli` proporciona una interfaz de línea de comandos estructurada en subcomandos.

```text
Uso: ppf_cli <COMANDO>

Comandos:
  apply   Aplica un parche PPF a un archivo binario
  undo    Revierte un parche PPF3 de un archivo binario
  info    Muestra la información y metadatos de un parche PPF
  create  Crea un nuevo parche PPF3 comparando dos archivos binarios
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
Restaura el binario a su estado original previo a la aplicación del parche (requiere que el parche haya sido creado con datos de undo).

```bash
ppf_cli undo --bin juego.bin --patch traduccion.ppf
```

### 4. Crear un parche (`create`)
Compara el binario original y el modificado para generar un archivo PPF3 optimizado.

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
- `-o, --output <PATH>`: Ruta donde se escribirá el parche `.ppf`.
- `-u, --undo`: Incluye datos de restauración (permite usar el subcomando `undo`).
- `-x, --disable-validation`: Desactiva la verificación de integridad de bloque (Blockcheck).
- `-i, --imagetype <0|1>`: Tipo de imagen (`0` = BIN/RAW estándar [por defecto], `1` = GI / PrimoDVD).
- `-d, --description <TEXTO>`: Descripción de hasta 50 caracteres incrustada en la cabecera.
- `-f, --file-id <PATH>`: Archivo de texto opcional que se incrustará como metadatos `FILE_ID.DIZ` (hasta 3072 bytes).

---

## Uso como Biblioteca (`ppf_rust`)

Agrega la biblioteca a las dependencias de tu `Cargo.toml`:

```toml
[dependencies]
ppf_rust = { path = "../ppf_rust" } # o desde git `ppf_rust = { git = "https://github.com/plinkr/ppf_rust.git" }`
```

### Ejemplo 1: Inspeccionar metadatos de un parche

```rust
use ppf_rust::inspect_patch;

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

La biblioteca permite conectar callbacks para monitorizar el progreso en interfaces gráficas o consolas personalizadas.

```rust
use ppf_rust::{apply_patch, undo_patch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let on_start = |total_records: usize| {
        println!("Iniciando aplicación de {} registros...", total_records);
    };

    let on_progress = |records_done: usize| {
        // Invocado por cada registro aplicado
    };

    // Aplicar parche
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

### Ejemplo 3: Crear un parche PPF3 en memoria o disco

```rust
use ppf_rust::{create_patch, ImageType, PpfCreatorOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = PpfCreatorOptions {
        description: "Mi Parche v1.0".to_string(),
        image_type: ImageType::Bin,
        block_check: true,
        undo_data: true,
        file_id: Some(b"Metadatos extendidos del parche".to_vec()),
    };

    let progress_callback = |bytes_scanned: usize| {
        // Invocado por cada bloque procesado en paralelo
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

## Roadmap

- [x] Motor de parcheo de alto rendimiento PPF 1.0, 2.0 y 3.0.
- [x] Interfaz de línea de comandos (`ppf_cli`) con barras de progreso interactivas.
- [x] Generación concurrente de parches con Rayon y memoria mapeada.
- [ ] **Interfaz Gráfica (GUI)**: Implementación de una interfaz visual de escritorio.

---

## Licencia

Este proyecto está distribuido bajo los términos de la Licencia MIT. Consulta el archivo [LICENSE](LICENSE) para más detalles.

