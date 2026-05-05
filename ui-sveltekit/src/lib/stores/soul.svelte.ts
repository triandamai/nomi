import { chatApi } from '$lib/api/client';

export interface SoulVersion {
    id: string; // or number, depends on API
    version: number;
    change_reason: string;
    soul_content: string;
    created_at: string;
}

function createSoulStore() {
    let history = $state<SoulVersion[]>([]);
    let loading = $state(false);
    let error = $state<string | null>(null);

    return {
        get history() { return history; },
        get loading() { return loading; },
        get error() { return error; },

        async loadHistory(conversationId: string) {
            if (!conversationId) return;
            try {
                loading = true;
                error = null;
                const response = await chatApi.getSoulHistory(conversationId);
                if (response && response.data) {
                    history = response.data.sort((a: SoulVersion, b: SoulVersion) => b.version - a.version);
                } else {
                    history = [];
                }
            } catch (e: any) {
                error = e.message || 'Failed to load soul history';
                history = [];
            } finally {
                loading = false;
            }
        },

        async restoreVersion(conversationId: string, version: number) {
            try {
                await chatApi.restoreSoul(conversationId, version);
                await this.loadHistory(conversationId);
            } catch (e: any) {
                console.error('Failed to restore soul:', e);
                throw e;
            }
        }
    };
}

export const soulStore = createSoulStore();