import {
    defineCollection, z
} from "astro:content";
import { glob } from 'astro/loaders';

// Define the docs collection schema
const docs = defineCollection({
    loader: glob({
        pattern: `**/*.{md,mdx}`,
        base:    `./src/docs`,
    }),
    schema: z.object({
        title:            z.string(),
        description:      z.string().optional(),
        sidebar_position: z.number().optional(),
        slug:             z.string().optional(),
        draft:            z.boolean().default(false),
    }),
});

// Export the collections
export const collections = {
    docs,
};
