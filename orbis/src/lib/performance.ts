/**
 * Performance optimization utilities
 * - Expression caching for schema interpolation
 * - Memoization helpers
 */

/**
 * Expression cache with LRU eviction
 * Using module-level Map for simpler implementation
 */
const MAX_CACHE_SIZE = 1000;

// 1 minute TTL
const EXPRESSION_CACHE_TTL_MS = 60000;

interface CacheEntry {
    value:       unknown
    timestamp:   number
    accessCount: number
}

// Module-level cache
const ExpressionCacheMap = new Map<string, CacheEntry>();

/**
 * Generate a cache key from expression and state
 */
function generateCacheKey(expression: string, state: unknown): string {
    const stateKey = typeof state === `object` && state !== null
        ? JSON.stringify(state)
        : String(state);
    return `${ expression }::${ stateKey }`;
}

/**
 * Get a cached expression result
 */
export function getCachedExpression(expression: string, state: unknown): unknown | undefined {
    const key = generateCacheKey(expression, state);
    const entry = ExpressionCacheMap.get(key);

    if (!entry) {
        return undefined;
    }

    // Check TTL
    const now = Date.now();
    if (now - entry.timestamp > EXPRESSION_CACHE_TTL_MS) {
        ExpressionCacheMap.delete(key);
        return undefined;
    }

    // Update access count and move to end (LRU)
    entry.accessCount += 1;
    ExpressionCacheMap.delete(key);
    ExpressionCacheMap.set(key, entry);

    return entry.value;
}

/**
 * Set a cached expression result
 */
export function setCachedExpression(expression: string, state: unknown, value: unknown): void {
    const key = generateCacheKey(expression, state);

    // Evict if at capacity
    if (ExpressionCacheMap.size >= MAX_CACHE_SIZE) {
        const oldestKey = ExpressionCacheMap.keys().next().value;
        if (oldestKey !== undefined) {
            ExpressionCacheMap.delete(oldestKey);
        }
    }

    ExpressionCacheMap.set(key, {
        value,
        timestamp:   Date.now(),
        accessCount: 1,
    });
}

/**
 * Clear the expression cache
 */
export function clearExpressionCache(): void {
    ExpressionCacheMap.clear();
}

/**
 * Get expression cache stats
 */
export function getExpressionCacheStats(): {
    size:    number
    maxSize: number
} {
    return {
        size:    ExpressionCacheMap.size,
        maxSize: MAX_CACHE_SIZE,
    };
}

/**
 * Cached expression interpolation
 */
export function cachedInterpolate(
    expression: string,
    state: Record<string, unknown>,
    interpolate: (expr: string, state: Record<string, unknown>) => unknown
): unknown {
    // Check cache first
    const cached = getCachedExpression(expression, state);
    if (cached !== undefined) {
        return cached;
    }

    // Compute and cache
    const result = interpolate(expression, state);
    setCachedExpression(expression, state, result);
    return result;
}

/**
 * Create a memoized function with custom cache
 */
export function memoize<Args extends Array<unknown>, R>(
    fn: (...args: Args) => R,
    getKey?: (...args: Args) => string
): (...args: Args) => R {
    const cache = new Map<string, R>();

    return (...args: Args): R => {
        const key = getKey ? getKey(...args) : JSON.stringify(args);
        if (cache.has(key)) {
            return cache.get(key) as R;
        }

        const result = fn(...args);
        cache.set(key, result);
        return result;
    };
}

interface AsyncCacheEntry<R> {
    value:     R
    timestamp: number
}

/**
 * Create a memoized async function
 */
export function memoizeAsync<Args extends Array<unknown>, R>(
    fn: (...args: Args) => Promise<R>,
    getKey?: (...args: Args) => string,
    ttl?: number
): (...args: Args) => Promise<R> {
    const cache = new Map<string, AsyncCacheEntry<R>>();

    return async(...args: Args): Promise<R> => {
        const key = getKey ? getKey(...args) : JSON.stringify(args);
        const cached = cache.get(key);

        if (cached) {
            if (!ttl || Date.now() - cached.timestamp < ttl) {
                return cached.value;
            }
            cache.delete(key);
        }

        const result = await fn(...args);
        cache.set(key, {
            value:     result,
            timestamp: Date.now(),
        });
        return result;
    };
}

/**
 * Debounce function execution
 */
export function debounce<Args extends Array<unknown>>(
    fn: (...args: Args) => void,
    delay: number
): (...args: Args) => void {
    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    return (...args: Args): void => {
        if (timeoutId) {
            clearTimeout(timeoutId);
        }
        timeoutId = setTimeout(() => {
            fn(...args);
            timeoutId = null;
        }, delay);
    };
}

/**
 * Throttle function execution
 */
export function throttle<Args extends Array<unknown>>(
    fn: (...args: Args) => void,
    limit: number
): (...args: Args) => void {
    let lastCall = 0;
    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    return (...args: Args): void => {
        const now = Date.now();
        const remaining = limit - (now - lastCall);

        if (remaining <= 0) {
            if (timeoutId) {
                clearTimeout(timeoutId);
                timeoutId = null;
            }
            lastCall = now;
            fn(...args);
        }
        else {
            timeoutId ??= setTimeout(() => {
                lastCall = Date.now();
                timeoutId = null;
                fn(...args);
            }, remaining);
        }
    };
}

/**
 * RAF-based throttle for animations
 */
export function rafThrottle<Args extends Array<unknown>>(
    fn: (...args: Args) => void
): (...args: Args) => void {
    let rafId: number | null = null;
    let lastArgs: Args | null = null;

    return (...args: Args): void => {
        lastArgs = args;

        rafId ??= requestAnimationFrame(() => {
            if (lastArgs) {
                fn(...lastArgs);
            }
            rafId = null;
        });
    };
}

/**
 * Shallow comparison for objects
 */
export function shallowEqual<T extends Record<string, unknown>>(obj1: T, obj2: T): boolean {
    if (obj1 === obj2) {
        return true;
    }

    const keys1 = Object.keys(obj1);
    const keys2 = Object.keys(obj2);

    if (keys1.length !== keys2.length) {
        return false;
    }

    for (const key of keys1) {
        if (obj1[key] !== obj2[key]) {
            return false;
        }
    }

    return true;
}

/**
 * Deep comparison for objects (limited depth)
 */
const MAX_COMPARISON_DEPTH = 3;

export function deepEqual(a: unknown, b: unknown, depth = 0): boolean {
    if (a === b) {
        return true;
    }

    if (depth > MAX_COMPARISON_DEPTH) {
        return false;
    }

    if (typeof a !== typeof b) {
        return false;
    }

    if (typeof a !== `object` || a === null || b === null) {
        return false;
    }

    if (Array.isArray(a) && Array.isArray(b)) {
        if (a.length !== b.length) {
            return false;
        }
        return a.every((val, i) => deepEqual(val, b[i], depth + 1));
    }

    if (Array.isArray(a) || Array.isArray(b)) {
        return false;
    }

    const objA = a as Record<string, unknown>;
    const objB = b as Record<string, unknown>;

    const keysA = Object.keys(objA);
    const keysB = Object.keys(objB);

    if (keysA.length !== keysB.length) {
        return false;
    }

    return keysA.every((key) => deepEqual(objA[key], objB[key], depth + 1));
}
