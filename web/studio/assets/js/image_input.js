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

function toHex(bytes) {
    let value = '';
    for (const byte of bytes) value += byte.toString(16).padStart(2, '0');
    return value;
}

export async function decomposeNativeImage({ config, image, signal } = {}) {
    if (!config || !image || !(image.rgba instanceof Uint8Array)) throw new Error('A decoded RGBA image and native configuration are required.');
    const width = Number(image.width);
    const height = Number(image.height);
    const columns = Number(image.grid);
    const rows = columns;
    if (!Number.isInteger(width) || !Number.isInteger(height) || width < 1 || height < 1 || width > 256 || height > 256) {
        throw new Error('Native image dimensions must be within 1..256.');
    }
    if (image.rgba.length !== width * height * 4) throw new Error('Native image decomposition requires exact RGBA bytes.');
    if (!Number.isInteger(columns) || columns < 1 || columns > width || rows > height) {
        throw new Error('Native image grid must fit the image.');
    }
    const headers = {
        'Content-Type': 'application/octet-stream',
        Accept: 'application/octet-stream',
    };
    if (config.apiKey) headers.Authorization = 'Bearer ' + config.apiKey;
    const url = normalizeBaseUrl(config.baseUrl) + '/images/decompose?width=' + width + '&height=' + height + '&columns=' + columns + '&rows=' + rows;
    const response = await fetch(url, {
        method: 'POST',
        headers,
        body: image.rgba,
        signal,
        cache: 'no-store',
    });
    if (!response.ok) {
        const body = await response.text();
        let message = body;
        try {
            const payload = JSON.parse(body);
            message = payload && payload.error && payload.error.message || body;
        } catch (_error) {
            // A non-JSON error body is already the most faithful available message.
        }
        throw new Error(String(message || 'Native image decomposition failed with HTTP ' + response.status + '.'));
    }
    const header = (name) => {
        const value = response.headers.get(name);
        if (!value) throw invalidResponse();
        return value;
    };
    const schema = header('x-uor-image-schema');
    const sourceBlake3 = header('x-uor-source-blake3');
    const decompositionId = header('x-uor-decomposition-id');
    const responseWidth = Number(header('x-uor-image-width'));
    const responseHeight = Number(header('x-uor-image-height'));
    const responseColumns = Number(header('x-uor-grid-columns'));
    const responseRows = Number(header('x-uor-grid-rows'));
    const patchCount = Number(header('x-uor-patch-count'));
    if (schema !== 'uor-r4.image-decomposition/1'
        || !/^[0-9a-f]{64}$/.test(sourceBlake3)
        || decompositionId.length > 128
        || responseWidth !== width
        || responseHeight !== height
        || responseColumns !== columns
        || responseRows !== rows
        || patchCount !== columns * rows) {
        throw invalidResponse();
    }
    const canonical = new Uint8Array(await response.arrayBuffer());
    const magic = [82, 52, 73, 77, 71, 48, 48, 49];
    if (canonical.length < magic.length
        || canonical.length > 52 + 32 * 32 * 40
        || magic.some((byte, index) => canonical[index] !== byte)) {
        throw invalidResponse();
    }
    return {
        schema,
        canonical_hex: toHex(canonical),
        source_blake3: sourceBlake3,
        width,
        height,
        grid: { columns, rows },
        decomposition_id: decompositionId,
    };
}
