import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from '@tailwindcss/vite';
import path from "path";

const HOST = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async() => ({
    plugins: [
        tailwindcss(),
        react(),
    ],

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent Vite from obscuring rust errors
    clearScreen: false,

    // 2. tauri expects a fixed port, fail if that port is not available
    server:      {
        port:       1420,
        strictPort: true,
        host:       HOST || false,
        hmr:        HOST
      ? {
          protocol: `ws`,
          host:     HOST,
          port:     1421,
      }
      : undefined,
        watch: {
            // 3. tell Vite to ignore watching `src-tauri`
            ignored: [ `**/src-tauri/**` ],
        },
    },

    // Env variables starting with the item of `envPrefix` will be exposed in tauri's source code through `import.meta.env`.
    envPrefix: [
        `VITE_`,
        `TAURI_ENV_*`,
    ],
    build:     {
        target:    process.env.TAURI_PLATFORM == `windows` ? `chrome105` : `safari13`,

        // don't minify for debug builds
        minify:    !process.env.TAURI_DEBUG ? `esbuild` as const : false,

        // produce sourcemaps for debug builds
        sourcemap: Boolean(process.env.TAURI_DEBUG),

        // Code splitting configuration
        rollupOptions: {
            output: {
                manualChunks: {
                    // Vendor chunk for large dependencies
                    'vendor-react': [
                        `react`,
                        `react-dom`,
                        `react-router-dom`,
                    ],
                    'vendor-ui': [
                        `@radix-ui/react-dialog`,
                        `@radix-ui/react-dropdown-menu`,
                        `@radix-ui/react-select`,
                        `@radix-ui/react-tabs`,
                        `@radix-ui/react-tooltip`,
                    ],
                    'vendor-form': [
                        `@tanstack/react-form`,
                        `zod`,
                    ],
                    'vendor-icons': [
                        `lucide-react`,
                    ],
                },
            },
        },

        // Increase the warning limit since Tauri apps are single bundles
        chunkSizeWarningLimit: 600,
    },
    resolve: {
        alias: {
            "@": path.resolve(__dirname, `./src`),
        },
    },
}));
