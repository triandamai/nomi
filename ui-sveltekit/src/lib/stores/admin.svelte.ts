import { chatApi } from '$lib/api/client';

export type AdminConversation = {
    id: string;
    title: string | null;
    cumulative_tokens: number;
    max_token_usage: number;
    created_at: string;
};

export type AdminUser = {
    id: string;
    name: string | null;
    display_name: string | null;
    email: string | null;
    role: string;
    is_verified: boolean;
    created_at: string | null;
};

function createAdminStore() {
    let conversations = $state<AdminConversation[]>([]);
    let users = $state<AdminUser[]>([]);
    
    let convLoading = $state(false);
    let userLoading = $state(false);
    
    let convCursor = $state<string | null>(null);
    let userCursor = $state<string | null>(null);
    
    let hasMoreConvs = $state(true);
    let hasMoreUsers = $state(true);
    
    let userSearchQuery = $state('');

    return {
        get conversations() { return conversations; },
        get users() { return users; },
        get convLoading() { return convLoading; },
        get userLoading() { return userLoading; },
        get hasMoreConvs() { return hasMoreConvs; },
        get hasMoreUsers() { return hasMoreUsers; },
        get userSearchQuery() { return userSearchQuery; },
        set userSearchQuery(val: string) { userSearchQuery = val; },

        async fetchConversations(loadMore = false) {
            if (convLoading || (loadMore && !hasMoreConvs)) return;
            
            convLoading = true;
            try {
                const cursor = loadMore ? (convCursor ?? undefined) : undefined;
                const res = await chatApi.getAdminConversations(cursor);
                
                const newItems = res.data.items;
                convCursor = res.data.next_cursor;
                hasMoreConvs = !!convCursor;

                if (loadMore) {
                    conversations = [...conversations, ...newItems];
                } else {
                    conversations = newItems;
                }
            } catch (e) {
                console.error('Admin Store Error (Conversations):', e);
            } finally {
                convLoading = false;
            }
        },

        async fetchUsers(loadMore = false) {
            if (userLoading || (loadMore && !hasMoreUsers)) return;
            
            userLoading = true;
            try {
                const cursor = loadMore ? (userCursor ?? undefined) : undefined;
                const res = await chatApi.getAdminUsers(cursor, 20, userSearchQuery);
                
                const newItems = res.data.items;
                userCursor = res.data.next_cursor;
                hasMoreUsers = !!userCursor;

                if (loadMore) {
                    users = [...users, ...newItems];
                } else {
                    users = newItems;
                }
            } catch (e) {
                console.error('Admin Store Error (Users):', e);
            } finally {
                userLoading = false;
            }
        },

        async updateConversation(id: string, maxTokens: number) {
            try {
                await chatApi.updateAdminConversation(id, { max_token_usage: maxTokens });
                const idx = conversations.findIndex(c => c.id === id);
                if (idx !== -1) {
                    conversations[idx] = { ...conversations[idx], max_token_usage: maxTokens };
                }
            } catch (e) {
                console.error('Admin Store Error (Update):', e);
                throw e;
            }
        },

        resetUsers() {
            users = [];
            userCursor = null;
            hasMoreUsers = true;
        }
    };
}

export const adminStore = createAdminStore();
