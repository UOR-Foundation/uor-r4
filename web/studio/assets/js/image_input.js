import { readTextFiles } from './composer_input.js';
import { normalizeBaseUrl } from './studio_api.js';

export const IMAGE_LABEL = 'R4 patch input — experimental; not visual understanding';
export const MAX_IMAGE_DIMENSION = 256;
export const MAX_IMAGE_FILE_BYTES = 4 * 1024 * 1024;
const IMAGE_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'avif']);

function extension(name) {
    const parts = String(name || '').toLowerCase().split('.');
    return parts.length > 1 ? parts[parts.length - 1] : '';
}

export function isImageFile(file) {
    const type = String(file && file.type || '').toLowerCase();
    if (type === 'image/svg+xml' || extension(file && file.name) === 'svg') return false;
    return type.startsWith('image/') || IMAGE_EXTENSIONS.has(extension(file && file.name));
}

async function loadSource(file) {
    if (typeof createImageBitmap === 'function') {
        try {
            const source = await createImageBitmap(file);
            return {
                source,
                cleanup: () => {
                    if (typeof source.close === 'function') source.close();
                },
            };
        } catch (_error) {
            // Fall through to the HTML image decoder.
        }
    }
    const objectUrl = URL.createObjectURL(file);
    try {
        const image = new Image();
        image.src = objectUrl;
        await image.decode();
        return {
            source: image,
            cleanup: () => URL.revokeObjectURL(objectUrl),
        };
    } catch (error) {
        URL.revokeObjectURL(objectUrl);
        throw error;
    }
}

export async function decodeImageFile(file, options = {}) {
    if (!isImageFile(file)) throw new Error(String(file && file.name || 'attachment') + ' is not a supported raster image.');
    const size = Number(file && file.size || 0);
    if (size <= 0) throw new Error(String(file && file.name || 'image') + ' is empty.');
    const maxBytes = Number(options.maxBytes) > 0 ? Number(options.maxBytes) : MAX_IMAGE_FILE_BYTES;
    if (size > maxBytes) throw new Error(String(file && file.name || 'image') + ' exceeds the ' + maxBytes + '-byte image limit (4 MiB).');
    const maxDimension = Number(options.maxDimension) > 0 ? Math.min(MAX_IMAGE_DIMENSION, Math.floor(Number(options.maxDimension))) : MAX_IMAGE_DIMENSION;
    const loaded = await loadSource(file);
    try {
        const sourceWidth = Number(loaded.source.naturalWidth || loaded.source.width);
        const sourceHeight = Number(loaded.source.naturalHeight || loaded.source.height);
        if (!Number.isFinite(sourceWidth) || !Number.isFinite(sourceHeight) || sourceWidth <= 0 || sourceHeight <= 0) {
            throw new Error(String(file.name || 'image') + ' has no readable dimensions.');
        }
        const scale = Math.min(1, maxDimension / sourceWidth, maxDimension / sourceHeight);
        const width = Math.max(1, Math.round(sourceWidth * scale));
        const height = Math.max(1, Math.round(sourceHeight * scale));
        const grid = Number(options.grid) > 0 ? Math.floor(Number(options.grid)) : Math.min(2, width, height);
        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        const context = canvas.getContext('2d', { willReadFrequently: true });
        if (!context) throw new Error('Canvas RGBA decoding is unavailable in this browser.');
        context.clearRect(0, 0, width, height);
        context.drawImage(loaded.source, 0, 0, width, height);
        const pixels = context.getImageData(0, 0, width, height);
        const rgba = new Uint8Array(pixels.data.buffer.slice(0));
        if (rgba.length !== width * height * 4) throw new Error('Canvas returned an invalid RGBA buffer.');
        return {
            name: String(file.name || 'image'),
            kind: 'image',
            content: '',
            source: 'file',
            size,
            width,
            height,
            grid,
            rgba,
            previewUrl: canvas.toDataURL('image/png'),
            imageUrl: canvas.toDataURL('image/png'),
            mime: 'image/png',
        };
    } finally {
        loaded.cleanup();
    }
}

export async function readComposerFiles(files, options = {}) {
    const images = [];
    const text = [];
    for (const file of Array.from(files || [])) {
        (isImageFile(file) ? images : text).push(file);
    }
    const result = await readTextFiles(text, options);
    const attachments = result.attachments.slice();
    const errors = result.errors.slice();
    for (const file of images) {
        try {
            attachments.push(await decodeImageFile(file, options));
        } catch (error) {
            errors.push(String(file && file.name || 'image') + ' could not be decoded: ' + String(error && error.message || error));
        }
    }
    return { attachments, errors };
}

function invalidResponse() {
    return new Error('The native server returned an invalid decomposition response.');
}

export async function decomposeNativeImage({ config, image, signal } = {}) {
    if (!config || !image || !(image.rgba instanceof Uint8Array)) throw new Error('A decoded RGBA image and native configuration are required.');
    const headers = {
        'Content-Type': 'application/octet-stream',
        Accept: 'application/json',
        'X-UOR-Width': String(image.width),
        'X-UOR-Height': String(image.height),
        'X-UOR-Grid': String(image.grid),
    };
    if (config.apiKey) headers.Authorization = 'Bearer ' + config.apiKey;
    const response = await fetch(normalizeBaseUrl(config.baseUrl) + '/images/decompose', {
        method: 'POST',
        headers,
        body: image.rgba,
        signal,
    });
    const body = await response.text();
    let payload;
    try {
        payload = JSON.parse(body);
    } catch (_error) {
        throw new Error('The native decomposition response was not JSON (HTTP ' + response.status + ').');
    }
    if (!response.ok) {
        const message = payload && payload.error && payload.error.message;
        throw new Error(String(message || 'Native image decomposition failed with HTTP ' + response.status + '.'));
    }
    const valid = payload
        && payload.schema === 'uor-r4.image/1'
        && typeof payload.source_sha256 === 'string'
        && /^[0-9a-f]{64}$/.test(payload.source_sha256)
        && typeof payload.canonical_sha256 === 'string'
        && /^[0-9a-f]{64}$/.test(payload.canonical_sha256)
        && typeof payload.decomposition_id === 'string'
        && payload.decomposition_id.length > 0
        && typeof payload.canonical_prompt === 'string'
        && payload.canonical_prompt.startsWith('R4IMG1;sha256=')
        && payload.visual_understanding === false
        && Array.isArray(payload.patches);
    if (!valid) throw invalidResponse();
    return payload;
}
