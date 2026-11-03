import { afterEach, describe, expect, it, vi } from 'vitest';
// @ts-expect-error The plain-JavaScript image controller has no declaration file.
import { decodeImageFile, decomposeNativeImage, isImageFile, readComposerFiles, IMAGE_LABEL } from '../../assets/js/image_input.js';

function canvasFixture(width = 256, height = 128) {
    const context = {
        clearRect: vi.fn(),
        drawImage: vi.fn(),
        getImageData: vi.fn(() => ({ data: new Uint8ClampedArray(width * height * 4) })),
    };
    const canvas = {
        width: 0,
        height: 0,
        getContext: vi.fn(() => context),
        toDataURL: vi.fn(() => 'data:image/png;base64,preview'),
    };
    const original = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag: string) => tag === 'canvas' ? canvas as unknown as HTMLCanvasElement : original(tag));
    return { canvas, context };
}

afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
});

describe('image file decoding and composer intake', () => {
    it('downsamples proportionally, bounds RGBA/grid, and closes the bitmap', async () => {
        const { context } = canvasFixture();
        const bitmap = { width: 512, height: 256, close: vi.fn() };
        vi.stubGlobal('createImageBitmap', vi.fn().mockResolvedValue(bitmap));
        const file = new File([new Uint8Array(4)], 'photo.png', { type: 'image/png' });
        const image = await decodeImageFile(file);
        expect(image.width).toBe(256);
        expect(image.height).toBe(128);
        expect(image.grid).toBe(2);
        expect(image.rgba).toHaveLength(256 * 128 * 4);
        expect(image.previewUrl).toBe('data:image/png;base64,preview');
        expect(image.kind).toBe('image');
        expect(image.content).toBe('');
        expect(context.drawImage).toHaveBeenCalledWith(bitmap, 0, 0, 256, 128);
        expect(bitmap.close).toHaveBeenCalledOnce();
        expect(IMAGE_LABEL).toBe('R4 patch input — experimental; not visual understanding');
    });

    it('does not upscale tiny images and mixes text with images without sending', async () => {
        canvasFixture(1, 1);
        const bitmap = { width: 1, height: 1, close: vi.fn() };
        vi.stubGlobal('createImageBitmap', vi.fn().mockResolvedValue(bitmap));
        const result = await readComposerFiles([
            new File(['hello'], 'note.txt', { type: 'text/plain' }),
            new File([new Uint8Array(4)], 'pixel.png', { type: 'image/png' }),
        ]);
        expect(result.errors).toEqual([]);
        expect(result.attachments).toHaveLength(2);
        expect(result.attachments[0].kind).toBe('text');
        expect(result.attachments[1].kind).toBe('image');
        expect(result.attachments[1].grid).toBe(1);
        expect(result.attachments[1].rgba).toHaveLength(4);
        expect(isImageFile(new File(['x'], 'x.svg', { type: 'image/svg+xml' }))).toBe(false);
    });

    it('refuses oversized input and revokes fallback object URLs on decode failure', async () => {
        await expect(decodeImageFile({ name: 'huge.png', type: 'image/png', size: 4 * 1024 * 1024 + 1 })).rejects.toThrow('4 MiB');
        vi.stubGlobal('createImageBitmap', vi.fn().mockRejectedValue(new Error('unsupported')));
        const decode = vi.fn().mockRejectedValue(new Error('broken image'));
        vi.stubGlobal('Image', class { decode = decode; });
        const create = vi.fn(() => 'blob:fixture');
        const revoke = vi.fn();
        vi.stubGlobal('URL', Object.assign(URL, { createObjectURL: create, revokeObjectURL: revoke }));
        await expect(decodeImageFile(new File(['bad'], 'bad.png', { type: 'image/png' }))).rejects.toThrow('broken image');
        expect(revoke).toHaveBeenCalledWith('blob:fixture');
    });
});

describe('native image decomposition transport', () => {
    it('posts exact RGBA headers/body and returns the validated decomposition', async () => {
        const payload = {
            schema: 'uor-r4.image/1',
            source_sha256: 'a'.repeat(64),
            canonical_prompt: 'R4IMG1;sha256=' + 'a'.repeat(64) + ';size=1x1;grid=1;q12=304,-128,144,0',
            visual_understanding: false,
            patches: [],
        };
        const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify(payload), { status: 200 }));
        vi.stubGlobal('fetch', fetchMock);
        const rgba = new Uint8Array([12, 20, 28, 255]);
        const result = await decomposeNativeImage({
            config: { baseUrl: 'http://127.0.0.1:8765/v1/', apiKey: 'secret' },
            image: { rgba, width: 1, height: 1, grid: 1 },
        });
        expect(result).toEqual(payload);
        const [url, init] = fetchMock.mock.calls[0];
        expect(url).toBe('http://127.0.0.1:8765/v1/images/decompose');
        expect(init.method).toBe('POST');
        expect(init.headers).toMatchObject({
            'Content-Type': 'application/octet-stream',
            Authorization: 'Bearer secret',
            'X-UOR-Width': '1',
            'X-UOR-Height': '1',
            'X-UOR-Grid': '1',
        });
        expect(init.body).toBe(rgba);
    });

    it('surfaces native error envelopes and invalid success payloads', async () => {
        vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(JSON.stringify({ error: { message: 'dimension refused' } }), { status: 400 })));
        await expect(decomposeNativeImage({ config: { baseUrl: 'http://localhost/v1' }, image: { rgba: new Uint8Array(4), width: 1, height: 1, grid: 1 } })).rejects.toThrow('dimension refused');
        vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('{}', { status: 200 })));
        await expect(decomposeNativeImage({ config: { baseUrl: 'http://localhost/v1' }, image: { rgba: new Uint8Array(4), width: 1, height: 1, grid: 1 } })).rejects.toThrow('invalid decomposition');
    });
});
