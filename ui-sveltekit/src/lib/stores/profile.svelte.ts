import { useAvatar } from '$lib/utils';

export type Profile = {
    id: string;
    name: string;
    avatar: string;
    status: 'online' | 'offline' | 'idle';
};

function createProfileStore() {
    let currentUser = $state<Profile>({
        id: 'user-1',
        name: 'Admin User',
        avatar: useAvatar('Admin User'),
        status: 'online'
    });

    return {
        get currentUser() {
            return currentUser;
        },
        updateStatus(status: Profile['status']) {
            currentUser = { ...currentUser, status };
        }
    };
}

export const profileStore = createProfileStore();
