import type { APIRoute } from "astro";
import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';
import Fuse from "fuse.js";
import { FuseConfig } from "@/components/fuse.config";
import type { SearchEntry } from "@/components/SearchModal";

const FILENAME = fileURLToPath(import.meta.url);
const DIRNAME = path.dirname(FILENAME);

/**
 * Generate and return the search index as a JSON response
 */
export const GET: APIRoute = async() => {
    const keys = FuseConfig.keys ?? [];

    const index = Fuse.createIndex(
        keys,
        await buildSearchIndex()
    );

    return new Response(JSON.stringify(index.toJSON()), {
        headers: {
            'Content-Type': `application/json`,
        },
    });
};

/**
 * Strip MDX/JSX tags from content
 * @param content - The MDX file content
 */
function stripMDXTags(content: string): string {
    // Remove import statements
    content = content.replace(/^import\s+.*$/gm, ``);

    // Remove JSX/MDX components like <Component prop="value">...</Component>
    content = content.replace(/<[A-Z][a-zA-Z0-9]*\s+[^>]*>[\s\S]*?<\/[A-Z][a-zA-Z0-9]*>/g, ``);
    content = content.replace(/<[A-Z][a-zA-Z0-9]*\s+[^>]*\/>/g, ``);

    // Remove frontmatter that might have been left
    content = content.replace(/^---[\s\S]*?---/m, ``);

    // Remove code block markers but keep the code content
    content = content.replace(/```[a-z]*\n/g, ``);
    content = content.replace(/```/g, ``);

    // Remove HTML comments
    content = content.replace(/<!--[\s\S]*?-->/g, ``);

    // Clean up multiple newlines
    content = content.replace(/\n{3,}/g, `\n\n`);

    return content.trim();
}

/**
 * Extract frontmatter from MDX content
 * @param content - The MDX file content
 */
function extractFrontmatter(content: string): { title?: string
    description?:                                       string
    slug?:                                              string } {
    const frontmatterMatch = /^---\n([\s\S]*?)\n---/.exec(content);
    if (!frontmatterMatch) {
        return {};
    }

    const [ , frontmatter ] = frontmatterMatch;
    const title = (/title:\s*(.+)/.exec(frontmatter))?.[1]?.trim().replace(/^["']|["']$/g, ``);
    const description = (/description:\s*(.+)/.exec(frontmatter))?.[1]?.trim().replace(/^["']|["']$/g, ``);
    const slug = (/slug:\s*(.+)/.exec(frontmatter))?.[1]?.trim().replace(/^["']|["']$/g, ``);

    return {
        title,
        description,
        slug,
    };
}

/**
 * Convert a file path to a URL
 * @param filePath - The file path to convert
 * @param docsDir - The base directory for documentation files
 */
function filePathToUrl(filePath: string, docsDir: string): string {
    const relativePath = path.relative(docsDir, filePath);
    const urlPath = relativePath
        .replace(/\\/g, `/`)
        .replace(/\.mdx?$/, ``)
        .replace(/\/index$/, ``);

    return `/docs/${ urlPath }`;
}

/**
 * Recursively find all MDX files in a directory
 * @param dir - Directory to search
 */
function findMDXFiles(dir: string): Array<string> {
    const files: Array<string> = [];
    const entries = fs.readdirSync(dir, {
        withFileTypes: true,
    });

    for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...findMDXFiles(fullPath));
        }
        else if (entry.isFile() && /\.mdx?$/.test(entry.name)) {
            files.push(fullPath);
        }
    }

    return files;
}

/**
 * Build the search index from MDX files
 */
export function buildSearchIndex(): Array<SearchEntry> {
    // Try a few candidate paths so this works both during dev and during the static build

    // typical when running in source
    const c1 = path.join(DIRNAME, `..`, `docs`);

    // when build executor's cwd is project root
    const c2 = path.join(process.cwd(), `src`, `docs`);

    // fallback
    const c3 = path.join(process.cwd(), `docs`, `src`, `docs`);

    const candidates = [
        c1,
        c2,
        c3,
    ];

    let docsDir: string | null = null;
    for (const c of candidates) {
        if (fs.existsSync(c)) {
            docsDir = c;
            break;
        }
    }

    if (!docsDir) {
        throw new Error(`Cannot locate docs directory.`);
    }

    const mdxFiles = findMDXFiles(docsDir);
    const searchEntries: Array<SearchEntry> = [];

    for (const filePath of mdxFiles) {
        const content = fs.readFileSync(filePath, `utf-8`);
        const {
            title, description, slug,
        } = extractFrontmatter(content);

        // Remove frontmatter and extract clean content
        const contentWithoutFrontmatter = content.replace(/^---\n[\s\S]*?\n---\n/, ``);
        const cleanContent = stripMDXTags(contentWithoutFrontmatter);

        if (!title) {
            console.warn(`No title found for ${ filePath }, skipping`);
            continue;
        }

        const url = slug ? `/docs${ slug }` : filePathToUrl(filePath, docsDir);

        searchEntries.push({
            id:      url,
            title,
            description,
            content: cleanContent,
            url,
        });
    }

    return searchEntries;
}
