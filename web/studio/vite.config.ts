import preact from '@preact/preset-vite';
import { defineConfig } from 'vite';

export default defineConfig({
    plugins: [preact()],
    build: {
        target: 'es2022',
        outDir: 'dist',
        emptyOutDir: true,
        lib: {
            entry: 'src/main.ts',
            formats: ['es'],
            fileName: 'studio-foundation',
        },
    },
});
