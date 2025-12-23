import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import Fuse, { type FuseIndexRecords, type FuseResult, type FuseResultMatch } from 'fuse.js';
import * as Dialog from '@radix-ui/react-dialog';
import { FileText, Search, X } from 'lucide-react';
import { FuseConfig } from './fuse.config';

export interface SearchEntry {
    id:           string
    title:        string
    description?: string
    content:      string
    url:          string
}

interface SearchModalProps {
    open:    boolean
    onClose: () => void
}

type FuseIndex = { keys: readonly string[]; records: FuseIndexRecords; }

const SearchModal: React.FC<SearchModalProps> = ({ open, onClose }) => {
    const [query, setQuery] = useState(``);
    const [results, setResults] = useState<Array<FuseResult<SearchEntry>>>([]);
    const [searchData, setSearchData] = useState<Array<SearchEntry>>([]);
    const [searchIndex, setSearchIndex] = useState<FuseIndex | null>(null);
    const [selectedIndex, setSelectedIndex] = useState(0);
    const [isLoading, setIsLoading] = useState(true);
    const inputRef = useRef<HTMLInputElement>(null);

    // Load search index
    useEffect(() => {
        if (open && (searchIndex === null || searchData.length === 0)) {
            setIsLoading(true);
            
            Promise.all([
                fetch(`/data-index.json`)
                    .then((res) => res.json())
                    .then((data: FuseIndex) => {
                        setSearchIndex(data);
                    })
                    .catch((err) => {
                        console.error(`Failed to load search index:`, err);
                    }),
                fetch(`/data.json`)
                    .then((res) => res.json())
                    .then((data: Array<SearchEntry>) => {
                        setSearchData(data);
                    })
                    .catch((err) => {
                        console.error(`Failed to load search data:`, err);
                    })
            ])
            .then(() => {
                setIsLoading(false);
            })
            .catch(() => {
                setIsLoading(false);
            });
        }
    }, [open, searchIndex, searchData]);

    // Initialize Fuse.js with search configuration
    const fuse = useMemo(() => {
        if (searchData.length === 0 || searchIndex === null) {
            return null;
        }
        
        const parsed_index = Fuse.parseIndex<SearchEntry>(searchIndex);
        return new Fuse<SearchEntry>(searchData, FuseConfig, parsed_index);
    }, [searchData, searchIndex]);

    // Perform search
    useEffect(() => {
        if (!fuse || query.trim().length < 2) {
            setResults([]);
            setSelectedIndex(0);
            return;
        }

        const searchResults = fuse.search(query);
        setResults(searchResults.slice(0, 10));
        setSelectedIndex(0);
    }, [query, fuse]);

    // Focus input when modal opens
    useEffect(() => {
        if (open) {
            setTimeout(() => {
                inputRef.current?.focus();
            }, 100);
        } else {
            setQuery(``);
            setResults([]);
            setSelectedIndex(0);
        }
    }, [open]);

    // Handle keyboard navigation
    const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
        if (e.key === `ArrowDown`) {
            e.preventDefault();
            setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
        } else if (e.key === `ArrowUp`) {
            e.preventDefault();
            setSelectedIndex((prev) => Math.max(prev - 1, 0));
        } else if (e.key === `Enter` && results[selectedIndex]) {
            e.preventDefault();
            navigateToResult(results[selectedIndex].item);
        }
    }, [results, selectedIndex]);

    const navigateToResult = (item: SearchEntry) => {
        const url = item.url.endsWith("/") ? item.url : `${item.url}/`;
        window.location.href = url;
        onClose();
    };

    // Highlight matched text
    const highlightMatches = (text: string, matches?: readonly FuseResultMatch[]) => {
        if (!matches || matches.length === 0) {
            return text;
        }

        const indices = matches[0]?.indices || [];
        if (indices.length === 0) {
            return text;
        }

        const parts: React.ReactNode[] = [];
        let lastIndex = 0;

        indices.forEach(([start, end]: [number, number]) => {
            if (start > lastIndex) {
                parts.push(text.substring(lastIndex, start));
            }
            parts.push(
                <mark key={start} className="bg-yellow-200 dark:bg-yellow-900 rounded px-0.5">
                    {text.substring(start, end + 1)}
                </mark>,
            );
            lastIndex = end + 1;
        });

        if (lastIndex < text.length) {
            parts.push(text.substring(lastIndex));
        }

        return parts;
    };

    return (
        <Dialog.Root open={open} onOpenChange={onClose}>
            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 bg-black/50 z-50 animate-in fade-in-0" />
                <Dialog.Content
                    className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-2xl max-h-[80vh] bg-background border rounded-lg shadow-lg flex flex-col"
                    onKeyDown={handleKeyDown}
                >
                    {/* Search Input */}
                    <div className="flex items-center border-b px-4 py-3">
                        <Search className="mr-2 h-5 w-5 text-muted-foreground shrink-0" />
                        <input
                            ref={inputRef}
                            type="text"
                            placeholder="Search documentation..."
                            className="flex-1 bg-transparent outline-none text-sm"
                            value={query}
                            onChange={(e) => { setQuery(e.target.value); }}
                        />
                        {query && (
                            <button
                                type="button"
                                onClick={() => { setQuery(``); }}
                                className="ml-2 text-muted-foreground hover:text-foreground"
                            >
                                <X className="h-4 w-4" />
                            </button>
                        )}
                    </div>

                    {/* Results */}
                    <div className="flex-1 overflow-y-auto p-2">
                        {isLoading ? (
                            <div className="py-8 text-center text-sm text-muted-foreground">
                                Loading search index...
                            </div>
                        ) : query.trim().length < 2 ? (
                            <div className="py-8 text-center text-sm text-muted-foreground">
                                Type at least 2 characters to search
                            </div>
                        ) : results.length === 0 ? (
                            <div className="py-8 text-center text-sm text-muted-foreground">
                                No results found for &quot;{query}&quot;
                            </div>
                        ) : (
                            <div className="space-y-1">
                                {results.map((result, index) => {
                                    const titleMatches = result.matches?.filter((m: FuseResultMatch) => m.key === `title`);
                                    const descriptionMatches = result.matches?.filter((m: FuseResultMatch) => m.key === `description`);
                                    const contentMatches = result.matches?.filter((m: FuseResultMatch) => m.key === `content`);
                                    
                                    return (
                                        <button
                                            key={result.item.id}
                                            type="button"
                                            className={`cursor-pointer w-full text-left p-3 rounded-md transition-colors ${
                                                index === selectedIndex
                                                    ? `bg-accent`
                                                    : `hover:bg-accent/50`
                                            }`}
                                            onClick={() => { navigateToResult(result.item); }}
                                            onMouseEnter={() => { setSelectedIndex(index); }}
                                        >
                                            <div className="flex items-start gap-3">
                                                <FileText className="h-5 w-5 text-muted-foreground shrink-0 mt-0.5" />
                                                <div className="flex-1 min-w-0">
                                                    <div className="font-medium text-sm mb-1">
                                                        {highlightMatches(result.item.title, titleMatches)}
                                                    </div>
                                                    {result.item.description && (
                                                        <div className="text-xs text-muted-foreground line-clamp-1 mb-1">
                                                            {highlightMatches(result.item.description, descriptionMatches)}
                                                        </div>
                                                    )}
                                                    {contentMatches && contentMatches.length > 0 && (() => {
                                                        const firstMatchStart = contentMatches[0]?.indices[0]?.[0] || 0;
                                                        const firstMatchEnd = contentMatches[0]?.indices[0]?.[1] || 0;
                                                        const substringStart = Math.max(0, firstMatchStart - 40);
                                                        const substringEnd = Math.min(result.item.content.length, firstMatchEnd + 100);
                                                        const contentSubstring = result.item.content.substring(substringStart, substringEnd);

                                                        // Adjust match indices to be relative to the substring
                                                        const adjustedMatches: FuseResultMatch[] = contentMatches.map((match) => ({
                                                            ...match,
                                                            indices: match.indices
                                                                .map(([start, end]): [number, number] => [start - substringStart, end - substringStart])
                                                                .filter(([start, end]) => end >= 0 && start < contentSubstring.length) as unknown as readonly [number, number][],
                                                        }));

                                                        return (
                                                            <div className="text-xs text-muted-foreground line-clamp-2">
                                                                {highlightMatches(contentSubstring, adjustedMatches)}
                                                            </div>
                                                        );
                                                    })()}
                                                </div>
                                            </div>
                                        </button>
                                    );
                                })}
                            </div>
                        )}
                    </div>

                    {/* Footer */}
                    <div className="border-t px-4 py-2 flex items-center justify-between text-xs text-muted-foreground">
                        <div className="flex items-center gap-3">
                            <span className="flex items-center gap-1">
                                <kbd className="px-1.5 py-0.5 bg-muted rounded border">↑</kbd>
                                <kbd className="px-1.5 py-0.5 bg-muted rounded border">↓</kbd>
                                to navigate
                            </span>
                            <span className="flex items-center gap-1">
                                <kbd className="px-1.5 py-0.5 bg-muted rounded border">Enter</kbd>
                                to select
                            </span>
                        </div>
                        <span className="flex items-center gap-1">
                            <kbd className="px-1.5 py-0.5 bg-muted rounded border">Esc</kbd>
                            to close
                        </span>
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    );
};

export default SearchModal;
