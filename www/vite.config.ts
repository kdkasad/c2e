import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
    plugins: [
        tailwindcss(),
    ],
    server: {
        fs: {
            // Allow serving files from the repository root
            allow: ['..']
        }
    }
})
