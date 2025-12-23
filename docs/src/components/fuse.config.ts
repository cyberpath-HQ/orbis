import type { IFuseOptions } from "fuse.js";
import type { SearchEntry } from "./SearchModal";

export const FuseConfig: IFuseOptions<SearchEntry> = {
    keys:              [
        {
            name:   `title`,
            weight: 3,
        },
        {
            name:   `description`,
            weight: 2,
        },
        {
            name:   `content`,
            weight: 1,
        },
    ],
    isCaseSensitive:    false,
    useExtendedSearch:  false,
    ignoreDiacritics:   true,
    threshold:          0.3,
    includeScore:       true,
    includeMatches:     true,
    minMatchCharLength: 2,
    ignoreLocation:     true,
    shouldSort:         true,
};
