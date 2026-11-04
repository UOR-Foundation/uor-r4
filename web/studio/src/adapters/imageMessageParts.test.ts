import { describe, expect, it } from 'vitest';
import source from '../../index.html?raw';

function factory(apiConfig: unknown, attachments: unknown[]) {
    const from = source.indexOf('function isNativeLoopback');
    const to = source.indexOf('async function sendViaApi', from);
    expect(from).toBeGreaterThan(0);
    expect(to).toBeGreaterThan(from);
    const create = new Function('app', 'window', `${source.slice(from, to)} return apiMessages;`);
    return create(
        { apiConfig },
        { __studioUi: { getAttachments: () => attachments } },
    ) as (conversation: { messages: { role: string; content: string }[] }) => { role: string; content: unknown }[];
}

const canonicalHex = '5234494d4730303166697874757265';
const image = {
    name: 'pixel.png',
    kind: 'image',
    content: '',
    imageUrl: 'data:image/png;base64,preview',
    decomposition: {
        schema: 'uor-r4.image-decomposition/1',
        canonical_hex: canonicalHex,
        source_blake3: 'a'.repeat(64),
        width: 1,
        height: 1,
        grid: { columns: 1, rows: 1 },
        decomposition_id: 'r4img-test',
    },
};

describe('API image content parts', () => {
    it('appends only canonical R4 parts for native loopback', () => {
        const apiMessages = factory({ id: 'native-loopback', ephemeral: true }, [image]);
        const messages = apiMessages({ messages: [{ role: 'user', content: 'Hi' }] });
        expect(messages[0].content).toEqual([
            { type: 'text', text: 'Hi' },
            { type: 'uor_r4_image', ...image.decomposition },
        ]);
    });

    it('uses ordinary image_url only for custom providers', () => {
        const apiMessages = factory({ id: 'custom', ephemeral: false }, [image]);
        const messages = apiMessages({ messages: [{ role: 'user', content: 'Hi' }] });
        expect(messages[0].content).toEqual([
            { type: 'text', text: 'Hi' },
            { type: 'image_url', image_url: { url: image.imageUrl } },
        ]);
    });

    it('uses ordinary image_url for a reserved ID without the ephemeral native flag', () => {
        const apiMessages = factory({ id: 'native-loopback', ephemeral: false }, [image]);
        const messages = apiMessages({ messages: [{ role: 'user', content: 'Hi' }] });
        expect(messages[0].content).toEqual([
            { type: 'text', text: 'Hi' },
            { type: 'image_url', image_url: { url: image.imageUrl } },
        ]);
    });

    it('refuses placeholder images that must be reattached', () => {
        const apiMessages = factory({ id: 'native-loopback', ephemeral: true }, [{ name: 'old.png', kind: 'image', content: '', reattachRequired: true }]);
        expect(() => apiMessages({ messages: [{ role: 'user', content: 'Hi' }] })).toThrow(/reattach/i);
    });
});
