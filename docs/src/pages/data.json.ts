import type { APIRoute } from "astro";
import { buildSearchIndex } from "./data-index.json";

/**
 * Generate and return the search index as a JSON response
 */
export const GET: APIRoute = async() => {
    const data = await buildSearchIndex();

    return new Response(JSON.stringify(data), {
        headers: {
            'Content-Type': `application/json`,
        },
    });
};
