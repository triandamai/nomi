import {chatApi} from '$lib/api/client';
import {goto} from '$app/navigation';

export type Profile = {
    id: string;
    external_id: string;
    display_name: string | null;
    avatar_url: string | null;
    status: 'online' | 'offline' | 'idle';
    role?: string;
};

export type ModelInfo = {
    agent_model: string;
    rag_embedding: string;
    media_classification: string;
    media_analyze: string;
};

export const AUTH_SESS_ID = "13ca7c4f"
export const AUTH_USER_ID = "12ca6c4f"

export function setSession(session: string, user_id: string) {
    sessionStorage.setItem(AUTH_SESS_ID, btoa(session));
    sessionStorage.setItem(AUTH_USER_ID, btoa(user_id));
}

export function getSession() {
    const token = sessionStorage.getItem(AUTH_SESS_ID);
    const user_id = sessionStorage.getItem(AUTH_USER_ID);

    return [token ? atob(token) : null, user_id ? atob(user_id) : null]
}

export function removeSession(){
    sessionStorage.removeItem(AUTH_SESS_ID);
    sessionStorage.removeItem(AUTH_USER_ID);
}

function createProfileStore() {
    let currentUser = $state<Profile | null>(null);
    let modelInfo = $state<ModelInfo | null>(null);
    let userChannels = $state<any[]>([]);
    let userConversations = $state<any[]>([]);
    let loading = $state(false);

    async function fetchProfile() {
        if (loading) return;
        loading = true;
        try {
            const [profileResponse, modelResponse] = await Promise.all([
                chatApi.getProfile(),
                chatApi.getModelInfo()
            ]);
            currentUser = {
                ...profileResponse.data,
                status: 'online'
            };
            modelInfo = modelResponse.data;
        } catch (e) {
            console.error('Failed to fetch profile or model info', e);
            // If unauthorized, redirect to login
            if (typeof window !== 'undefined' && (e as any).message?.includes('Unauthorized')) {
                goto('/login');
            }
        } finally {
            loading = false;
        }
    }

    async function updateProfile(displayName: string) {
        try {
            await chatApi.updateProfile(displayName);
            if (currentUser) {
                currentUser.display_name = displayName;
            }
        } catch (e) {
            console.error('Failed to update profile', e);
            throw e;
        }
    }

    async function fetchUserConnections() {
        try {
            const [channels, conversations] = await Promise.all([
                chatApi.getChannels(),
                chatApi.getConversations()
            ]);
            userChannels = channels.data;
            userConversations = conversations.data;
        } catch (e) {
            console.error('Failed to fetch user connections', e);
        }
    }

    async function logout() {
        try {
            await chatApi.logout();
        } catch (e) {
            console.error('Logout API failed', e);
        } finally {
            if (typeof sessionStorage !== 'undefined') {
                removeSession()
            }
            currentUser = null;
            modelInfo = null;
            goto('/login');
        }
    }

    return {
        get currentUser() {
            return currentUser;
        },
        get modelInfo() {
            return modelInfo;
        },
        get userChannels() {
            return userChannels;
        },
        get userConversations() {
            return userConversations;
        },
        get loading() {
            return loading;
        },
        fetchProfile,
        updateProfile,
        fetchUserConnections,
        logout,
        updateStatus(status: Profile['status']) {
            if (currentUser) {
                currentUser = {...currentUser, status};
            }
        }
    };
}

export const profileStore = createProfileStore();
