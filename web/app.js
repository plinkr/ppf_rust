// PPF Rust Patcher - Web Client Application

// Worker Management
const worker = new Worker('worker.js', { type: 'module' });
let nextMessageId = 1;
const pendingRequests = new Map();

worker.onmessage = (event) => {
    const { id, type, success, info, binBytes, patchBytes, error, percent, message } = event.data;
    const req = pendingRequests.get(id);
    if (!req) return;

    if (type === 'progress') {
        if (req.onProgress) {
            req.onProgress({ percent, message });
        }
    } else if (type === 'complete') {
        pendingRequests.delete(id);
        if (success) {
            req.resolve({ info, binBytes, patchBytes });
        } else {
            req.reject(new Error(error || 'Operation failed'));
        }
    }
};

function sendWorkerMessage(type, payload, transferables = [], onProgress = null) {
    return new Promise((resolve, reject) => {
        const id = nextMessageId++;
        pendingRequests.set(id, { resolve, reject, onProgress });
        worker.postMessage({ id, type, payload }, transferables);
    });
}

// Utility Functions
function formatFileSize(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function triggerDownload(buffer, filename, mimeType = 'application/octet-stream') {
    const blob = new Blob([buffer], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    setTimeout(() => URL.revokeObjectURL(url), 60000);
}

function generatePatchedFilename(originalName) {
    const lastDot = originalName.lastIndexOf('.');
    if (lastDot === -1) {
        return originalName + '_patched';
    }
    return originalName.substring(0, lastDot) + '_patched' + originalName.substring(lastDot);
}

function generateRestoredFilename(originalName) {
    const lastDot = originalName.lastIndexOf('.');
    if (lastDot === -1) {
        return originalName + '_restored';
    }
    return originalName.substring(0, lastDot) + '_restored' + originalName.substring(lastDot);
}

// Dropzone Handler Helper
function setupDropzone(options) {
    const {
        dropzoneEl,
        inputEl,
        emptyEl,
        selectedEl,
        nameEl,
        sizeEl,
        clearEl,
        browseBtn,
        onFileSelected,
        onFileCleared
    } = options;

    const handleFile = (file) => {
        if (!file) return;
        emptyEl.classList.add('hidden');
        selectedEl.classList.remove('hidden');
        nameEl.textContent = file.name;
        sizeEl.textContent = formatFileSize(file.size);
        if (onFileSelected) onFileSelected(file);
    };

    dropzoneEl.addEventListener('dragover', (e) => {
        e.preventDefault();
        dropzoneEl.classList.add('dragover');
    });

    dropzoneEl.addEventListener('dragleave', () => {
        dropzoneEl.classList.remove('dragover');
    });

    dropzoneEl.addEventListener('drop', (e) => {
        e.preventDefault();
        dropzoneEl.classList.remove('dragover');
        if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
            inputEl.files = e.dataTransfer.files;
            handleFile(e.dataTransfer.files[0]);
        }
    });

    dropzoneEl.addEventListener('click', (e) => {
        if (e.target !== clearEl && !selectedEl.contains(e.target)) {
            inputEl.click();
        }
    });

    if (browseBtn) {
        browseBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            inputEl.click();
        });
    }

    inputEl.addEventListener('change', () => {
        if (inputEl.files && inputEl.files.length > 0) {
            handleFile(inputEl.files[0]);
        }
    });

    clearEl.addEventListener('click', (e) => {
        e.stopPropagation();
        inputEl.value = '';
        selectedEl.classList.add('hidden');
        emptyEl.classList.remove('hidden');
        if (onFileCleared) onFileCleared();
    });
}

// DOM Elements
// Tab Navigation
const tabButtons = document.querySelectorAll('.tab-btn');
const tabContents = document.querySelectorAll('.tab-content');

// Tab 1: Apply / Undo
const applyBinDropzone = document.getElementById('apply-bin-dropzone');
const applyBinInput = document.getElementById('apply-bin-input');
const applyBinEmpty = document.getElementById('apply-bin-empty');
const applyBinSelected = document.getElementById('apply-bin-selected');
const applyBinName = document.getElementById('apply-bin-name');
const applyBinSize = document.getElementById('apply-bin-size');
const applyBinClear = document.getElementById('apply-bin-clear');
const applyBinBrowse = document.getElementById('apply-bin-browse');
const applyOutputName = document.getElementById('apply-output-name');

const applyPatchDropzone = document.getElementById('apply-patch-dropzone');
const applyPatchInput = document.getElementById('apply-patch-input');
const applyPatchEmpty = document.getElementById('apply-patch-empty');
const applyPatchSelected = document.getElementById('apply-patch-selected');
const applyPatchName = document.getElementById('apply-patch-name');
const applyPatchSize = document.getElementById('apply-patch-size');
const applyPatchClear = document.getElementById('apply-patch-clear');
const applyPatchBrowse = document.getElementById('apply-patch-browse');

const applyInfoPlaceholder = document.getElementById('apply-info-placeholder');
const applyInfoCard = document.getElementById('apply-info-card');
const applyInfoVersion = document.getElementById('apply-info-version');
const applyInfoDescription = document.getElementById('apply-info-description');
const applyInfoImagetypeRow = document.getElementById('apply-info-imagetype-row');
const applyInfoImagetype = document.getElementById('apply-info-imagetype');
const applyInfoBlockcheckRow = document.getElementById('apply-info-blockcheck-row');
const applyInfoBlockcheck = document.getElementById('apply-info-blockcheck');
const applyInfoUndoRow = document.getElementById('apply-info-undo-row');
const applyInfoUndo = document.getElementById('apply-info-undo');
const applyInfoFileid = document.getElementById('apply-info-fileid');

const btnApplyPatch = document.getElementById('btn-apply-patch');
const btnUndoPatch = document.getElementById('btn-undo-patch');
const applyProgressBox = document.getElementById('apply-progress-box');
const applyProgressTitle = document.getElementById('apply-progress-title');
const applyProgressPercent = document.getElementById('apply-progress-percent');
const applyProgressFill = document.getElementById('apply-progress-fill');
const applyProgressDetail = document.getElementById('apply-progress-detail');
const applyStatusBanner = document.getElementById('apply-status-banner');

// Tab 2: Create Patch
const createOrigDropzone = document.getElementById('create-orig-dropzone');
const createOrigInput = document.getElementById('create-orig-input');
const createOrigEmpty = document.getElementById('create-orig-empty');
const createOrigSelected = document.getElementById('create-orig-selected');
const createOrigName = document.getElementById('create-orig-name');
const createOrigSize = document.getElementById('create-orig-size');
const createOrigClear = document.getElementById('create-orig-clear');
const createOrigBrowse = document.getElementById('create-orig-browse');

const createModDropzone = document.getElementById('create-mod-dropzone');
const createModInput = document.getElementById('create-mod-input');
const createModEmpty = document.getElementById('create-mod-empty');
const createModSelected = document.getElementById('create-mod-selected');
const createModName = document.getElementById('create-mod-name');
const createModSize = document.getElementById('create-mod-size');
const createModClear = document.getElementById('create-mod-clear');
const createModBrowse = document.getElementById('create-mod-browse');

const createOutputName = document.getElementById('create-output-name');
const createDescInput = document.getElementById('create-desc-input');
const createDescCounter = document.getElementById('create-desc-counter');
const createBlockCheck = document.getElementById('create-block-check');
const createUndoData = document.getElementById('create-undo-data');
const createDizInput = document.getElementById('create-diz-input');
const btnCreatePatch = document.getElementById('btn-create-patch');

const createProgressBox = document.getElementById('create-progress-box');
const createProgressTitle = document.getElementById('create-progress-title');
const createProgressPercent = document.getElementById('create-progress-percent');
const createProgressFill = document.getElementById('create-progress-fill');
const createProgressDetail = document.getElementById('create-progress-detail');
const createStatusBanner = document.getElementById('create-status-banner');

// Tab 3: Info & DIZ
const infoPatchDropzone = document.getElementById('info-patch-dropzone');
const infoPatchInput = document.getElementById('info-patch-input');
const infoPatchEmpty = document.getElementById('info-patch-empty');
const infoPatchSelected = document.getElementById('info-patch-selected');
const infoPatchName = document.getElementById('info-patch-name');
const infoPatchSize = document.getElementById('info-patch-size');
const infoPatchClear = document.getElementById('info-patch-clear');
const infoPatchBrowse = document.getElementById('info-patch-browse');

const infoPlaceholder = document.getElementById('info-placeholder');
const infoGrid = document.getElementById('info-grid');
const infoVersion = document.getElementById('info-version');
const infoDescription = document.getElementById('info-description');
const infoImagetypeRow = document.getElementById('info-imagetype-row');
const infoImagetype = document.getElementById('info-imagetype');
const infoBlockcheckRow = document.getElementById('info-blockcheck-row');
const infoBlockcheck = document.getElementById('info-blockcheck');
const infoUndoRow = document.getElementById('info-undo-row');
const infoUndo = document.getElementById('info-undo');
const infoFileidStatus = document.getElementById('info-fileid-status');

const infoDizCard = document.getElementById('info-diz-card');
const infoDizViewer = document.getElementById('info-diz-viewer');
const btnCopyDiz = document.getElementById('btn-copy-diz');

// State Variables
let state = {
    applyBinFile: null,
    applyPatchFile: null,
    applyPatchInfo: null,
    createOrigFile: null,
    createModFile: null,
    infoPatchFile: null,
    infoPatchInfo: null,
    isProcessing: false
};

// Tab Switching
tabButtons.forEach((btn) => {
    btn.addEventListener('click', () => {
        const targetTabId = btn.dataset.tab;
        tabButtons.forEach((b) => {
            b.classList.toggle('active', b === btn);
            b.setAttribute('aria-selected', b === btn ? 'true' : 'false');
        });
        tabContents.forEach((c) => {
            c.classList.toggle('active', c.id === targetTabId);
        });
    });
});

// Update Apply / Undo Button States
function updateApplyButtonStates() {
    const hasBin = state.applyBinFile !== null;
    const hasPatch = state.applyPatchFile !== null && state.applyPatchInfo !== null;
    const canApply = hasBin && hasPatch && !state.isProcessing;
    const canUndo = canApply && state.applyPatchInfo.has_undo;

    btnApplyPatch.disabled = !canApply;
    btnUndoPatch.disabled = !canUndo;
}

// Update Create Button State
function updateCreateButtonState() {
    const hasOrig = state.createOrigFile !== null;
    const hasMod = state.createModFile !== null;
    const canCreate = hasOrig && hasMod && !state.isProcessing;

    btnCreatePatch.disabled = !canCreate;
}

// Render Patch Info into Elements
function renderPatchInfo(info, elements) {
    const {
        versionEl,
        descEl,
        imageTypeRow,
        imageTypeEl,
        blockCheckRow,
        blockCheckEl,
        undoRow,
        undoEl,
        fileIdEl
    } = elements;

    let verText = 'PPF 1.0';
    if (info.version === 'V2') verText = 'PPF 2.0 (Blockcheck format)';
    else if (info.version === 'V3') verText = 'PPF 3.0 (Advanced format)';
    else if (info.version === 'V1') verText = 'PPF 1.0 (Legacy format)';
    versionEl.textContent = verText;

    descEl.textContent = info.description || '(No description)';

    if (info.version === 'V3') {
        imageTypeRow.classList.remove('hidden');
        imageTypeEl.textContent = info.image_type === 'Gi'
            ? 'GI (Global Image / PrimoDVD)'
            : 'BIN (Standard RAW / ISO)';

        blockCheckRow.classList.remove('hidden');
        if (info.block_check) {
            blockCheckEl.innerHTML = '<span class="badge badge-green">Enabled (1024-byte validation)</span>';
        } else {
            blockCheckEl.innerHTML = '<span class="badge badge-yellow">Disabled</span>';
        }

        undoRow.classList.remove('hidden');
        if (info.has_undo) {
            undoEl.innerHTML = '<span class="badge badge-green">Available (Reversible)</span>';
        } else {
            undoEl.innerHTML = '<span class="badge badge-yellow">Not available (One-way)</span>';
        }
    } else {
        imageTypeRow.classList.add('hidden');
        undoRow.classList.add('hidden');
        if (info.version === 'V2') {
            blockCheckRow.classList.remove('hidden');
            blockCheckEl.innerHTML = '<span class="badge badge-green">Enabled (PPF2 Checksum)</span>';
        } else {
            blockCheckRow.classList.add('hidden');
        }
    }

    if (info.file_id) {
        fileIdEl.innerHTML = '<span class="badge badge-green">Embedded in trailer</span>';
    } else {
        fileIdEl.innerHTML = '<span class="dim">None</span>';
    }
}

// Setup Tab 1: Apply & Undo Dropzones
setupDropzone({
    dropzoneEl: applyBinDropzone,
    inputEl: applyBinInput,
    emptyEl: applyBinEmpty,
    selectedEl: applyBinSelected,
    nameEl: applyBinName,
    sizeEl: applyBinSize,
    clearEl: applyBinClear,
    browseBtn: applyBinBrowse,
    onFileSelected: (file) => {
        state.applyBinFile = file;
        applyOutputName.value = generatePatchedFilename(file.name);
        applyStatusBanner.classList.add('hidden');
        updateApplyButtonStates();
    },
    onFileCleared: () => {
        state.applyBinFile = null;
        updateApplyButtonStates();
    }
});

setupDropzone({
    dropzoneEl: applyPatchDropzone,
    inputEl: applyPatchInput,
    emptyEl: applyPatchEmpty,
    selectedEl: applyPatchSelected,
    nameEl: applyPatchName,
    sizeEl: applyPatchSize,
    clearEl: applyPatchClear,
    browseBtn: applyPatchBrowse,
    onFileSelected: async (file) => {
        state.applyPatchFile = file;
        state.applyPatchInfo = null;
        applyStatusBanner.classList.add('hidden');
        applyInfoPlaceholder.textContent = 'Inspecting patch file...';
        applyInfoPlaceholder.classList.remove('hidden');
        applyInfoCard.classList.add('hidden');

        try {
            const buffer = await file.arrayBuffer();
            const res = await sendWorkerMessage('inspect', { patchBytes: buffer });
            state.applyPatchInfo = res.info;

            applyInfoPlaceholder.classList.add('hidden');
            applyInfoCard.classList.remove('hidden');
            renderPatchInfo(res.info, {
                versionEl: applyInfoVersion,
                descEl: applyInfoDescription,
                imageTypeRow: applyInfoImagetypeRow,
                imageTypeEl: applyInfoImagetype,
                blockCheckRow: applyInfoBlockcheckRow,
                blockCheckEl: applyInfoBlockcheck,
                undoRow: applyInfoUndoRow,
                undoEl: applyInfoUndo,
                fileIdEl: applyInfoFileid
            });
        } catch (err) {
            applyInfoPlaceholder.textContent = `Failed to parse patch: ${err.message}`;
            applyInfoPlaceholder.classList.remove('hidden');
            applyInfoCard.classList.add('hidden');
        }
        updateApplyButtonStates();
    },
    onFileCleared: () => {
        state.applyPatchFile = null;
        state.applyPatchInfo = null;
        applyInfoPlaceholder.textContent = 'Select or drag & drop a .ppf patch file above to inspect its metadata.';
        applyInfoPlaceholder.classList.remove('hidden');
        applyInfoCard.classList.add('hidden');
        updateApplyButtonStates();
    }
});

// Setup Tab 2: Create Patch Dropzones
setupDropzone({
    dropzoneEl: createOrigDropzone,
    inputEl: createOrigInput,
    emptyEl: createOrigEmpty,
    selectedEl: createOrigSelected,
    nameEl: createOrigName,
    sizeEl: createOrigSize,
    clearEl: createOrigClear,
    browseBtn: createOrigBrowse,
    onFileSelected: (file) => {
        state.createOrigFile = file;
        createStatusBanner.classList.add('hidden');
        if (state.createModFile && state.createOrigFile.size !== state.createModFile.size) {
            showBanner(
                createStatusBanner,
                'error',
                `Size Mismatch: Original file is ${formatFileSize(state.createOrigFile.size)}, but Modified file is ${formatFileSize(state.createModFile.size)}. Both files must have the exact same size.`
            );
        }
        updateCreateButtonState();
    },
    onFileCleared: () => {
        state.createOrigFile = null;
        updateCreateButtonState();
    }
});

setupDropzone({
    dropzoneEl: createModDropzone,
    inputEl: createModInput,
    emptyEl: createModEmpty,
    selectedEl: createModSelected,
    nameEl: createModName,
    sizeEl: createModSize,
    clearEl: createModClear,
    browseBtn: createModBrowse,
    onFileSelected: (file) => {
        state.createModFile = file;
        createStatusBanner.classList.add('hidden');
        if (state.createOrigFile && state.createOrigFile.size !== state.createModFile.size) {
            showBanner(
                createStatusBanner,
                'error',
                `Size Mismatch: Original file is ${formatFileSize(state.createOrigFile.size)}, but Modified file is ${formatFileSize(state.createModFile.size)}. Both files must have the exact same size.`
            );
        }
        updateCreateButtonState();
    },
    onFileCleared: () => {
        state.createModFile = null;
        updateCreateButtonState();
    }
});

// Description Character Counter
createDescInput.addEventListener('input', () => {
    createDescCounter.textContent = `${createDescInput.value.length}/50`;
});

// Setup Tab 3: Info Dropzone
setupDropzone({
    dropzoneEl: infoPatchDropzone,
    inputEl: infoPatchInput,
    emptyEl: infoPatchEmpty,
    selectedEl: infoPatchSelected,
    nameEl: infoPatchName,
    sizeEl: infoPatchSize,
    clearEl: infoPatchClear,
    browseBtn: infoPatchBrowse,
    onFileSelected: async (file) => {
        state.infoPatchFile = file;
        infoPlaceholder.textContent = 'Inspecting patch file...';
        infoPlaceholder.classList.remove('hidden');
        infoGrid.classList.add('hidden');
        infoDizCard.classList.add('hidden');

        try {
            const buffer = await file.arrayBuffer();
            const res = await sendWorkerMessage('inspect', { patchBytes: buffer });
            state.infoPatchInfo = res.info;

            infoPlaceholder.classList.add('hidden');
            infoGrid.classList.remove('hidden');
            renderPatchInfo(res.info, {
                versionEl: infoVersion,
                descEl: infoDescription,
                imageTypeRow: infoImagetypeRow,
                imageTypeEl: infoImagetype,
                blockCheckRow: infoBlockcheckRow,
                blockCheckEl: infoBlockcheck,
                undoRow: infoUndoRow,
                undoEl: infoUndo,
                fileIdEl: infoFileidStatus
            });

            if (res.info.file_id) {
                infoDizViewer.textContent = res.info.file_id;
                infoDizCard.classList.remove('hidden');
            } else {
                infoDizCard.classList.add('hidden');
            }
        } catch (err) {
            infoPlaceholder.textContent = `Failed to parse patch: ${err.message}`;
            infoPlaceholder.classList.remove('hidden');
            infoGrid.classList.add('hidden');
            infoDizCard.classList.add('hidden');
        }
    },
    onFileCleared: () => {
        state.infoPatchFile = null;
        state.infoPatchInfo = null;
        infoPlaceholder.textContent = 'Select or drag & drop a PPF patch file above to inspect its metadata and embedded release information.';
        infoPlaceholder.classList.remove('hidden');
        infoGrid.classList.add('hidden');
        infoDizCard.classList.add('hidden');
    }
});

// Copy DIZ Button
btnCopyDiz.addEventListener('click', async () => {
    const text = infoDizViewer.textContent;
    if (!text) return;
    try {
        await navigator.clipboard.writeText(text);
        const originalLabel = btnCopyDiz.textContent;
        btnCopyDiz.textContent = 'Copied!';
        setTimeout(() => {
            btnCopyDiz.textContent = originalLabel;
        }, 2000);
    } catch (err) {
        console.error('Failed to copy DIZ text:', err);
    }
});

// Banner Display Helper
function showBanner(bannerEl, type, message) {
    bannerEl.className = `status-banner ${type}`;
    bannerEl.textContent = message;
    bannerEl.classList.remove('hidden');
}

// Action: Apply Patch
btnApplyPatch.addEventListener('click', async () => {
    if (!state.applyBinFile || !state.applyPatchFile || state.isProcessing) return;

    state.isProcessing = true;
    updateApplyButtonStates();
    applyStatusBanner.classList.add('hidden');
    applyProgressBox.classList.remove('hidden');
    applyProgressTitle.textContent = 'Applying PPF patch...';
    applyProgressPercent.textContent = '0%';
    applyProgressFill.style.width = '0%';
    applyProgressDetail.textContent = 'Reading binary image into memory...';

    try {
        const patchBuffer = await state.applyPatchFile.arrayBuffer();
        const binBuffer = await state.applyBinFile.arrayBuffer();

        const onProgress = ({ percent, message }) => {
            applyProgressPercent.textContent = `${percent.toFixed(1)}%`;
            applyProgressFill.style.width = `${percent}%`;
            applyProgressDetail.textContent = message;
        };

        const res = await sendWorkerMessage(
            'apply',
            { patchBytes: patchBuffer, binBytes: binBuffer },
            [binBuffer],
            onProgress
        );

        applyProgressPercent.textContent = '100%';
        applyProgressFill.style.width = '100%';
        applyProgressDetail.textContent = 'Completed!';

        const outName = applyOutputName.value.trim() || generatePatchedFilename(state.applyBinFile.name);
        triggerDownload(res.binBytes, outName);

        showBanner(
            applyStatusBanner,
            'success',
            `Patch successfully applied! Downloaded "${outName}". Description: "${res.info.description}"`
        );
    } catch (err) {
        showBanner(applyStatusBanner, 'error', `Failed to apply patch: ${err.message}`);
    } finally {
        state.isProcessing = false;
        applyProgressBox.classList.add('hidden');
        updateApplyButtonStates();
    }
});

// Action: Undo Patch
btnUndoPatch.addEventListener('click', async () => {
    if (!state.applyBinFile || !state.applyPatchFile || state.isProcessing) return;

    state.isProcessing = true;
    updateApplyButtonStates();
    applyStatusBanner.classList.add('hidden');
    applyProgressBox.classList.remove('hidden');
    applyProgressTitle.textContent = 'Reversing PPF patch...';
    applyProgressPercent.textContent = '0%';
    applyProgressFill.style.width = '0%';
    applyProgressDetail.textContent = 'Reading binary image into memory...';

    try {
        const patchBuffer = await state.applyPatchFile.arrayBuffer();
        const binBuffer = await state.applyBinFile.arrayBuffer();

        const onProgress = ({ percent, message }) => {
            applyProgressPercent.textContent = `${percent.toFixed(1)}%`;
            applyProgressFill.style.width = `${percent}%`;
            applyProgressDetail.textContent = message;
        };

        const res = await sendWorkerMessage(
            'undo',
            { patchBytes: patchBuffer, binBytes: binBuffer },
            [binBuffer],
            onProgress
        );

        applyProgressPercent.textContent = '100%';
        applyProgressFill.style.width = '100%';
        applyProgressDetail.textContent = 'Completed!';

        const outName = generateRestoredFilename(state.applyBinFile.name);
        triggerDownload(res.binBytes, outName);

        showBanner(
            applyStatusBanner,
            'success',
            `Patch successfully reversed! Downloaded original binary as "${outName}".`
        );
    } catch (err) {
        showBanner(applyStatusBanner, 'error', `Failed to undo patch: ${err.message}`);
    } finally {
        state.isProcessing = false;
        applyProgressBox.classList.add('hidden');
        updateApplyButtonStates();
    }
});

// Action: Create Patch
btnCreatePatch.addEventListener('click', async () => {
    if (!state.createOrigFile || !state.createModFile || state.isProcessing) return;

    if (state.createOrigFile.size !== state.createModFile.size) {
        showBanner(
            createStatusBanner,
            'error',
            `Size Mismatch: Original file is ${formatFileSize(state.createOrigFile.size)}, but Modified file is ${formatFileSize(state.createModFile.size)}. Both files must have the exact same size.`
        );
        return;
    }

    state.isProcessing = true;
    updateCreateButtonState();
    createStatusBanner.classList.add('hidden');
    createProgressBox.classList.remove('hidden');
    createProgressTitle.textContent = 'Scanning differences...';
    createProgressPercent.textContent = '0%';
    createProgressFill.style.width = '0%';
    createProgressDetail.textContent = 'Reading original and modified binaries...';

    try {
        const origBuffer = await state.createOrigFile.arrayBuffer();
        const modBuffer = await state.createModFile.arrayBuffer();

        const imageType = document.querySelector('input[name="create-image-type"]:checked')?.value || 'bin';
        const description = createDescInput.value.trim() || undefined;
        const blockCheck = createBlockCheck.checked;
        const undoData = createUndoData.checked;
        const fileId = createDizInput.value.trim() || undefined;

        const options = {
            description,
            imageType,
            blockCheck,
            undoData,
            fileId
        };

        const onProgress = ({ percent, message }) => {
            createProgressPercent.textContent = `${percent.toFixed(1)}%`;
            createProgressFill.style.width = `${percent}%`;
            createProgressDetail.textContent = message;
        };

        const res = await sendWorkerMessage(
            'create',
            { origBytes: origBuffer, modBytes: modBuffer, options },
            [origBuffer, modBuffer],
            onProgress
        );

        createProgressPercent.textContent = '100%';
        createProgressFill.style.width = '100%';
        createProgressDetail.textContent = 'Patch created!';

        const outName = createOutputName.value.trim() || 'patch.ppf';
        triggerDownload(res.patchBytes, outName, 'application/octet-stream');

        showBanner(
            createStatusBanner,
            'success',
            `PPF3 patch successfully generated and downloaded as "${outName}" (${formatFileSize(res.patchBytes.byteLength)}).`
        );
    } catch (err) {
        showBanner(createStatusBanner, 'error', `Failed to create patch: ${err.message}`);
    } finally {
        state.isProcessing = false;
        createProgressBox.classList.add('hidden');
        updateCreateButtonState();
    }
});
