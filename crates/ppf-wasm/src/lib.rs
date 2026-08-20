use ppf_core::{
    ImageType, PatchAction, PatchInfo, PpfCreatorOptions, create_patch_slice, inspect_patch_slice,
    patch_slice,
};
use serde::Deserialize;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::prelude::*;

struct SendSyncFn(js_sys::Function);
unsafe impl Send for SendSyncFn {}
unsafe impl Sync for SendSyncFn {}

impl SendSyncFn {
    fn call_progress(&self, done: f64, total: f64) {
        let _ = self
            .0
            .call2(&JsValue::NULL, &JsValue::from(done), &JsValue::from(total));
    }
}

fn js_err(err: impl std::fmt::Display) -> JsValue {
    JsValue::from(js_sys::Error::new(&err.to_string()))
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WasmCreatorOptions {
    pub description: Option<String>,
    pub image_type: Option<String>,
    pub block_check: Option<bool>,
    pub undo_data: Option<bool>,
    pub file_id: Option<String>,
}

#[wasm_bindgen]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Inspects a PPF patch from an in-memory byte slice and returns patch metadata.
#[wasm_bindgen]
pub fn inspect_patch(patch_bytes: &[u8]) -> Result<JsValue, JsValue> {
    let info = inspect_patch_slice(patch_bytes).map_err(js_err)?;
    serde_wasm_bindgen::to_value(&info).map_err(js_err)
}

fn process_patch_wasm(
    patch_bytes: &[u8],
    target_bytes: &mut [u8],
    progress_cb: Option<js_sys::Function>,
    action: PatchAction,
) -> Result<JsValue, JsValue> {
    let total_records = Rc::new(Cell::new(0usize));
    let done_records = Rc::new(Cell::new(0usize));

    let total_start = total_records.clone();
    let on_start = progress_cb.as_ref().map(|cb| {
        let cb = cb.clone();
        move |total: usize| {
            total_start.set(total);
            let _ = cb.call2(
                &JsValue::NULL,
                &JsValue::from(0usize),
                &JsValue::from(total as f64),
            );
        }
    });

    let total_progress = total_records.clone();
    let done_progress = done_records.clone();
    let on_progress = progress_cb.as_ref().map(|cb| {
        let cb = cb.clone();
        move |count: usize| {
            let done = done_progress.get() + count;
            done_progress.set(done);
            let total = total_progress.get();
            if total > 0 && (done == total || done.is_multiple_of(200)) {
                let _ = cb.call2(
                    &JsValue::NULL,
                    &JsValue::from(done as f64),
                    &JsValue::from(total as f64),
                );
            }
        }
    });

    let on_start_dyn = on_start.as_ref().map(|f| f as &dyn Fn(usize));
    let on_progress_dyn = on_progress.as_ref().map(|f| f as &dyn Fn(usize));

    let info: PatchInfo = patch_slice(
        patch_bytes,
        target_bytes,
        action,
        on_start_dyn,
        on_progress_dyn,
    )
    .map_err(js_err)?;

    serde_wasm_bindgen::to_value(&info).map_err(js_err)
}

macro_rules! def_wasm_patch_fn {
    ($name:ident, $action:ident, $doc:expr) => {
        #[doc = $doc]
        #[wasm_bindgen]
        pub fn $name(
            patch_bytes: &[u8],
            target_bytes: &mut [u8],
            progress_cb: Option<js_sys::Function>,
        ) -> Result<JsValue, JsValue> {
            process_patch_wasm(patch_bytes, target_bytes, progress_cb, PatchAction::$action)
        }
    };
}

def_wasm_patch_fn!(
    apply_patch,
    Apply,
    "Applies a PPF patch to a target binary byte slice in-place."
);
def_wasm_patch_fn!(
    undo_patch,
    Undo,
    "Reverses/undoes a PPF3 patch on a target binary byte slice in-place."
);

/// Creates a PPF3 patch comparing original and modified binary byte slices.
#[wasm_bindgen]
pub fn create_patch(
    orig_bytes: &[u8],
    mod_bytes: &[u8],
    options: JsValue,
    progress_cb: Option<js_sys::Function>,
) -> Result<js_sys::Uint8Array, JsValue> {
    let wasm_opts: WasmCreatorOptions = if options.is_undefined() || options.is_null() {
        WasmCreatorOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options).map_err(js_err)?
    };

    let img_type = match wasm_opts.image_type.as_deref() {
        Some("gi") | Some("GI") => ImageType::Gi,
        _ => ImageType::Bin,
    };

    let core_opts = PpfCreatorOptions {
        description: wasm_opts
            .description
            .filter(|d| !d.trim().is_empty())
            .unwrap_or_else(|| ppf_core::DEFAULT_DESCRIPTION.to_string()),
        image_type: img_type,
        block_check: wasm_opts.block_check.unwrap_or(true),
        undo_data: wasm_opts.undo_data.unwrap_or(false),
        file_id: wasm_opts.file_id.map(|s| s.into_bytes()),
    };

    let total_bytes = orig_bytes.len();
    let done_bytes = AtomicUsize::new(0);

    let on_progress = progress_cb.as_ref().map(|cb| {
        let send_cb = SendSyncFn(cb.clone());
        move |bytes: usize| {
            let done = done_bytes.fetch_add(bytes, Ordering::Relaxed) + bytes;
            send_cb.call_progress(done as f64, total_bytes as f64);
        }
    });

    let mut output = Vec::new();
    create_patch_slice(
        orig_bytes,
        mod_bytes,
        &mut output,
        &core_opts,
        on_progress
            .as_ref()
            .map(|f| f as &(dyn Fn(usize) + Sync + Send)),
    )
    .map_err(js_err)?;

    Ok(js_sys::Uint8Array::from(&output[..]))
}
