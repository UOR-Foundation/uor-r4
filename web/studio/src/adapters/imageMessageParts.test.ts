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

const canonical = 'R4IMG1;sha256=' + 'a'.repeat(64) + ';size=1x1;grid=1;q12=304,-128,144,0';
const image = {
    name: 'pixel.png',
    kind: 'image',
    content: canonical,
    imageUrl: 'data:image/png;base64,preview',
    decomposition: {
        schema: 'uor-r4.image/1',
        source_sha256: 'a'.repeat(64),
        canonical_sha256: 'b'.repeat(64),
        canonical_prompt: canonical,
    },
};

describe('API image content parts', () => {
    it('appends only canonical R4 parts for native loopback', () => {
        const apiMessages = factory({ id: 'native-loopback', ephemeral: true }, [image]);
        const messages = apiMessages({ messages: [{ role: 'user', content: 'Hi' }] });
        expect(messages[0].content).toEqual([
            { type: 'text', text: 'Hi' },
            { type: 'uor_r4_image', uor_r4_image: image.decomposition },
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
