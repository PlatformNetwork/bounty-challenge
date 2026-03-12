/**
 * src/terminal/autoRepliesModel.ts
 * 
 * Model for managing Terminal Auto Replies.
 * Fixes bug where whitespace-only search input was treated as a valid filter.
 */

export interface AutoReplyRule {
    id: string;
    matchPattern: string;
    response: string;
    isDefault?: boolean;
}

export class AutoRepliesModel {
    private allRules: AutoReplyRule[] = [];

    constructor(rules: AutoReplyRule[] = []) {
        this.allRules = rules;
    }

    public getRules(): AutoReplyRule[] {
        return this.allRules;
    }

    /**
     * Filters the list of rules based on a search query.
     * 
     * FIX: Trims the input query. If the query is empty or consists only of whitespace,
     * it returns the full list of rules instead of filtering for spaces.
     * 
     * @param query The raw input string from the search field.
     * @returns The filtered list of rules.
     */
    public filterRules(query: string): AutoReplyRule[] {
        const normalizedQuery = query.trim();

        // If the trimmed query is empty, treat it as "no filter" (show all rules)
        if (normalizedQuery.length === 0) {
            return this.allRules;
        }

        // Perform the actual filtering logic
        return this.allRules.filter(rule => {
            const matchFound = rule.matchPattern.toLowerCase().includes(normalizedQuery.toLowerCase());
            const responseFound = rule.response.toLowerCase().includes(normalizedQuery.toLowerCase());
            return matchFound || responseFound;
        });
    }
}
