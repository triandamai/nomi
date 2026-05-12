import { useAvatar } from '$lib/utils';
import { chatApi } from '$lib/api/client';
import { goto } from '$app/navigation';

export type Profile = {
    id: string;
    external_id: string;
    display_name: string | null;
    avatar_url: string | null;
    status: 'online' | 'offline' | 'idle';
    role?: string;
};

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
            console.log(currentUser)
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
                currentUser = { ...currentUser, status };
            }
        }
    };
}

export const profileStore = createProfileStore();
