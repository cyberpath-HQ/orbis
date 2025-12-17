// @ts-check

import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";
import react from "@astrojs/react";
import sitemap from "@astrojs/sitemap";

// https://astro.build/config
export default defineConfig({
    integrations: [
        react(),
        sitemap({
            filter: (page) => !page.includes('/api/'),
        }),
        mdx({
            extendMarkdownConfig: true,
            gfm:                  true,
        }),
    ],
    vite: {
        plugins: [
            tailwindcss({
                optimize: true,
            }),
        ],
    },
    build: {
        assets: `assets`,
    },
    markdown: {
        shikiConfig: {
            theme: 'github-dark',
            wrap:  true,
        },
    },
    site:          `https://orbis.cyberpath-hq.com`,
    base:          `/`,
    trailingSlash: `always`,
});
