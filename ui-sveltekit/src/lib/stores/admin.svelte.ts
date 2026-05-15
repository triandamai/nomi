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

export type AdminUserDetail = {
    user: AdminUser;
    channels: Array<{
        id: string;
        channel_type: string;
        external_id: string;
        external_chat_id: string;
        conversation_title: string | null;
    }>;
    conversations: Array<{
        conversation_id: string;
        title: string | null;
        joined_at: string | null;
    }>;
};

function createAdminStore() {
    let conversations = $state<AdminConversation[]>([]);
    let users = $state<AdminUser[]>([]);
    let selectedUserDetail = $state<AdminUserDetail | null>(null);
    
    let convLoading = $state(false);
    let userLoading = $state(false);
    let detailLoading = $state(false);
    
    let convCursor = $state<string | null>(null);
    let userCursor = $state<string | null>(null);
    
    let hasMoreConvs = $state(true);
    let hasMoreUsers = $state(true);
    
    let userSearchQuery = $state('');

    return {
        get conversations() { return conversations; },
        get users() { return users; },
        get selectedUserDetail() { return selectedUserDetail; },
        get convLoading() { return convLoading; },
        get userLoading() { return userLoading; },
        get detailLoading() { return detailLoading; },
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

        async fetchUserDetail(id: string) {
            detailLoading = true;
            try {
                const res = await chatApi.getAdminUserDetail(id);
                selectedUserDetail = res.data;
            } catch (e) {
                console.error('Admin Store Error (Detail):', e);
            } finally {
                detailLoading = false;
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

        async updateAdminUser(id: string, updates: any) {
            try {
                await chatApi.updateAdminUser(id, updates);
                if (selectedUserDetail?.user.id === id) {
                    selectedUserDetail.user = { ...selectedUserDetail.user, ...updates };
                }
                const idx = users.findIndex(u => u.id === id);
                if (idx !== -1) {
                    users[idx] = { ...users[idx], ...updates };
                }
            } catch (e) {
                console.error('Admin Store Error (User Update):', e);
                throw e;
            }
        },

        async deleteAdminUser(id: string) {
            try {
                await chatApi.deleteAdminUser(id);
                users = users.filter(u => u.id !== id);
                if (selectedUserDetail?.user.id === id) {
                    selectedUserDetail = null;
                }
            } catch (e) {
                console.error('Admin Store Error (User Delete):', e);
                throw e;
            }
        },

        resetUsers() {
            users = [];
            userCursor = null;
            hasMoreUsers = true;
            selectedUserDetail = null;
        }
    };
}

export const adminStore = createAdminStore();
