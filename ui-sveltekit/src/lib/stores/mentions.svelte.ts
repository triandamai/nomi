import { chatApi } from '$lib/api/client';

class MentionStore {
    // Svelte 5 reactive cache
    private cache = $state<Record<string, string>>({});
    private loading = new Set<string>();

    /**
     * Resolves the display name of a mentioned external ID.
     * Reactively triggers an API lookup in the background if the ID is uncached.
     */
    getDisplayName(externalId: string): string {
        if (this.cache[externalId]) {
            return this.cache[externalId];
        }

        if (!this.loading.has(externalId)) {
            this.loading.add(externalId);
            this.fetchDisplayName(externalId);
        }

        return `@${externalId}`; // Fallback while loading
    }

    private async fetchDisplayName(externalId: string) {
        try {
            const response = await chatApi.lookupExternalUser(externalId);
            if (response && response.data) {
                // Cache the display name from the database
                this.cache[externalId] = response.data.display_name;
            } else {
                // If not found in channels, keep the raw external ID format
                this.cache[externalId] = `@${externalId}`;
            }
        } catch (e) {
            console.error(`MentionStore: Failed to fetch display name for ${externalId}`, e);
            this.cache[externalId] = `@${externalId}`;
        }
    }
}

export const mentionStore = new MentionStore();
