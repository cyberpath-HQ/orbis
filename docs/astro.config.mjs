// @ts-check

import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";

import react from "@astrojs/react";
import sitemap from "@astrojs/sitemap";

// https://astro.build/config
export default defineConfig({
    vite: {
        plugins: [
            tailwindcss({
                optimize: true,
            }),
            react(),
            sitemap(),
            mdx({
                extendMarkdownConfig: true,
                gfm:                  true,
            }),
        ],
    },
    build:        {
        assets: `assets`,
    },
    site:          `https://orbis.cyberpath-hq.com`,
    base:          `/`,
    trailingSlash: `always`,
});
