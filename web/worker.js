import init, { inspect_patch, apply_patch, undo_patch, create_patch, get_app_version } from './pkg/ppf_wasm.js';

let wasmReady = false;
let wasmInitPromise = null;

async function ensureWasm() {
    if (!wasmReady) {
        if (!wasmInitPromise) {
            wasmInitPromise = init().then(() => {
                wasmReady = true;
            });
        }
        await wasmInitPromise;
    }
}

self.onmessage = async (event) => {
    const { id, type, payload } = event.data;

    try {
        await ensureWasm();

        if (type === 'inspect') {
            const patchBytes = new Uint8Array(payload.patchBytes);
            const info = inspect_patch(patchBytes);
            self.postMessage({ id, type: 'complete', success: true, info });
        } else if (type === 'apply') {
            const patchBytes = new Uint8Array(payload.patchBytes);
            const targetBytes = new Uint8Array(payload.binBytes);

            const progressCb = (done, total) => {
                const percent = total > 0 ? (done / total) * 100 : 0;
                self.postMessage({
                    id,
                    type: 'progress',
                    done,
                    total,
                    percent,
                    message: `Applying records: ${Math.round(done)} / ${Math.round(total)} (${percent.toFixed(1)}%)`
                });
            };

            const info = apply_patch(patchBytes, targetBytes, progressCb);
            self.postMessage(
                {
                    id,
                    type: 'complete',
                    success: true,
                    info,
                    binBytes: targetBytes.buffer
                },
                [targetBytes.buffer]
            );
        } else if (type === 'undo') {
            const patchBytes = new Uint8Array(payload.patchBytes);
            const targetBytes = new Uint8Array(payload.binBytes);

            const progressCb = (done, total) => {
                const percent = total > 0 ? (done / total) * 100 : 0;
                self.postMessage({
                    id,
                    type: 'progress',
                    done,
                    total,
                    percent,
                    message: `Undoing records: ${Math.round(done)} / ${Math.round(total)} (${percent.toFixed(1)}%)`
                });
            };

            const info = undo_patch(patchBytes, targetBytes, progressCb);
            self.postMessage(
                {
                    id,
                    type: 'complete',
                    success: true,
                    info,
                    binBytes: targetBytes.buffer
                },
                [targetBytes.buffer]
            );
        } else if (type === 'create') {
            const origBytes = new Uint8Array(payload.origBytes);
            const modBytes = new Uint8Array(payload.modBytes);
            const options = payload.options;

            const progressCb = (doneBytes, totalBytes) => {
                const percent = totalBytes > 0 ? (doneBytes / totalBytes) * 100 : 0;
                const mbDone = (doneBytes / (1024 * 1024)).toFixed(1);
                const mbTotal = (totalBytes / (1024 * 1024)).toFixed(1);
                self.postMessage({
                    id,
                    type: 'progress',
                    done: doneBytes,
                    total: totalBytes,
                    percent,
                    message: `${mbDone} MB / ${mbTotal} MB (${percent.toFixed(1)}%)`
                });
            };

            const patchUint8 = create_patch(origBytes, modBytes, options, progressCb);
            self.postMessage(
                {
                    id,
                    type: 'complete',
                    success: true,
                    patchBytes: patchUint8.buffer
                },
                [patchUint8.buffer]
            );
        } else if (type === 'get_version') {
            const version = get_app_version();
            self.postMessage({ id, type: 'complete', success: true, version });
        } else {
            throw new Error(`Unknown action type: ${type}`);
        }
    } catch (err) {
        const errorMsg = err?.message || String(err);
        self.postMessage({ id, type: 'complete', success: false, error: errorMsg });
    }
};
