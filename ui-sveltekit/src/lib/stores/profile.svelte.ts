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

function createProfileStore() {
    let currentUser = $state<Profile | null>(null);
    let loading = $state(false);

    async function fetchProfile() {
        if (loading) return;
        loading = true;
        try {
            const response = await chatApi.getProfile();
            currentUser = {
                ...response.data,
                status: 'online'
            };
        } catch (e) {
            console.error('Failed to fetch profile', e);
            // If unauthorized, redirect to login
            if (typeof window !== 'undefined' && (e as any).message?.includes('Unauthorized')) {
                goto('/login');
            }
        } finally {
            loading = false;
        }
    }

    async function logout() {
        try {
            await chatApi.logout();
        } catch (e) {
            console.error('Logout API failed', e);
        } finally {
            if (typeof sessionStorage !== 'undefined') {
                sessionStorage.removeItem('auth_token');
                sessionStorage.removeItem('user_id');
            }
            currentUser = null;
            goto('/login');
        }
    }

    return {
        get currentUser() {
            return currentUser;
        },
        get loading() {
            return loading;
        },
        fetchProfile,
        logout,
        updateStatus(status: Profile['status']) {
            if (currentUser) {
                currentUser = {...currentUser, status};
            }
        }
    };
}

export const profileStore = createProfileStore();
