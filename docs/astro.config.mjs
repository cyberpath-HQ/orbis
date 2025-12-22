// @ts-check

import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";
import react from "@astrojs/react";
import sitemap from "@astrojs/sitemap";
import rehypeAutolinkHeadings from "rehype-autolink-headings";
import rehypeSlug from "rehype-slug";

// https://astro.build/config
export default defineConfig({
    integrations: [
        react(),
        sitemap({
            filter: (page) => !page.includes(`/api/`),
        }),
        mdx({
            extendMarkdownConfig: true,
            gfm:                  true,
            rehypePlugins:        [
                rehypeSlug,
                rehypeAutolinkHeadings,
            ],
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
            themes: {
                light: `github-light`,
                dark:  `github-dark`,
            },
            wrap: false,
        },
        rehypePlugins: [
            rehypeSlug,
            rehypeAutolinkHeadings,
        ],
    },
    site:          `https://orbis.cyberpath-hq.com`,
    base:          `/`,
    trailingSlash: `always`,
});
