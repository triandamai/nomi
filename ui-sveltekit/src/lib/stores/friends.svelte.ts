import { chatApi } from '$lib/api/client';

export interface FriendProfile {
    id: string;
    name: string | null;
    display_name: string | null;
    email: string | null;
}

export interface FriendRequest {
    id: string;
    sender_id: string;
    receiver_id: string;
    sender_display_name: string | null;
    receiver_display_name: string | null;
    created_at: string;
}

function createFriendsStore() {
    let friends = $state<FriendProfile[]>([]);
    let incomingRequests = $state<FriendRequest[]>([]);
    let outgoingRequests = $state<FriendRequest[]>([]);
    let loading = $state(false);

    async function fetchFriends() {
        loading = true;
        try {
            const res = await chatApi.getFriends();
            friends = res.data || [];
        } catch (e) {
            console.error('Failed to fetch friends', e);
        } finally {
            loading = false;
        }
    }

    async function fetchRequests() {
        try {
            const res = await chatApi.getPendingRequests();
            incomingRequests = res.data?.incoming || [];
            outgoingRequests = res.data?.outgoing || [];
        } catch (e) {
            console.error('Failed to fetch pending requests', e);
        }
    }

    async function sendRequest(receiverId: string) {
        try {
            await chatApi.sendFriendRequest(receiverId);
            await fetchRequests();
        } catch (e) {
            console.error('Failed to send friend request', e);
            throw e;
        }
    }

    async function respondRequest(senderId: string, accept: boolean) {
        try {
            const res = await chatApi.respondFriendRequest(senderId, accept);
            await Promise.all([fetchFriends(), fetchRequests()]);
            return res.data; // will be conversation_id if accepted
        } catch (e) {
            console.error('Failed to respond to friend request', e);
            throw e;
        }
    }

    async function block(blockedUserId: string) {
        try {
            await chatApi.blockUser(blockedUserId);
            await Promise.all([fetchFriends(), fetchRequests()]);
        } catch (e) {
            console.error('Failed to block user', e);
            throw e;
        }
    }

    return {
        get friends() { return friends; },
        get incomingRequests() { return incomingRequests; },
        get outgoingRequests() { return outgoingRequests; },
        get loading() { return loading; },
        fetchFriends,
        fetchRequests,
        sendRequest,
        respondRequest,
        block
    };
}

export const friendsStore = createFriendsStore();
