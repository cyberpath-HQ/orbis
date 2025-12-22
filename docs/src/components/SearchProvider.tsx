import React, {
    useEffect, useState
} from 'react';
import SearchModal from './SearchModal';

export default function SearchProvider() {
    const [
        open,
        setOpen,
    ] = useState(false);

    useEffect(() => {
        function openSearch(): void {
            setOpen(true);
        }

        function handleKey(e: KeyboardEvent): void {
            if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === `k`) {
                e.preventDefault();
                setOpen(true);
            }
        }

        const btn = document.getElementById(`search-button`);
        btn?.addEventListener(`click`, openSearch);
        document.addEventListener(`keydown`, handleKey);

        return () => {
            btn?.removeEventListener(`click`, openSearch);
            document.removeEventListener(`keydown`, handleKey);
        };
    }, []);

    return <SearchModal open={open} onClose={() => setOpen(false)} />;
}
