import {eventBus} from '$lib/utils';
import type {Conversation} from "$lib/stores/conversation.svelte";

const BASE_URL = 'http://localhost:8000/api';
const CHANNEL_URL = 'http://localhost:8001/api';

export type Meta = {
    code: number,
    message: string
}
export type FieldError = {
    name: string,
    error_message: string[]
}
export type  ApiResponse<T> = {
    data: T,
    meta: Meta,
    errors: FieldError[] | null
}

export async function apiFetch<T>(endpoint: string, options: RequestInit = {}): Promise<ApiResponse<T>> {
    const token = typeof localStorage !== 'undefined' ? localStorage.getItem('auth_token') : null;
    const response = await fetch(`${BASE_URL}${endpoint}`, {
        ...options,
        headers: {
            'Content-Type': 'application/json',
            ...(token ? { 'Authorization': `Bearer ${token}` } : {}),
            ...options.headers
        }
    });

    if (!response.ok) {
        const error = await response.json().catch(() => ({message: 'An unknown error occurred'}));
        throw new Error(error.message || response.statusText);
    }

    return response.json();
}

export const chatApi = {
    sendMessage: (message: string, conversationId?: string) => {
        return apiFetch<{ reply: string }>('/chat', {
            method: 'POST',
            body: JSON.stringify({message, conversation_id: conversationId})
        });
    },

    getMessages: (conversationId: string, cursor?: string, limit: number = 20) => {
        const url = new URL(`${BASE_URL}/conversations/${conversationId}/messages`);
        if (cursor) url.searchParams.append('cursor', cursor);
        url.searchParams.append('limit', limit.toString());
        return apiFetch<any>(url.pathname.replace("/api", "") + url.search);
    },

    requestOtp: (externalId: string, channel: string) => {
        return apiFetch<any>('/auth/request-otp', {
            method: 'POST',
            body: JSON.stringify({external_id: externalId, channel})
        });
    },

    verifyOtp: (externalId: string, code: string) => {
        return apiFetch<{ access_token: string, user_id: string }>('/auth/verify-otp', {
            method: 'POST',
            body: JSON.stringify({external_id: externalId, code})
        });
    },

    streamChat: async (message: string, conversationId: string) => {
        const token = typeof localStorage !== 'undefined' ? localStorage.getItem('auth_token') : null;
        const response = await fetch(`${BASE_URL}/chat/stream`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                ...(token ? { 'Authorization': `Bearer ${token}` } : {}),
            },
            body: JSON.stringify({message, conversation_id: conversationId})
        });
        return response;
    },

    streamEvent() {
        // Based on gateway-rust/src/routes.rs, the SSE endpoint is /realtime
        const userId = typeof localStorage !== 'undefined' ? localStorage.getItem('user_id') : "9220f30e-b5cb-4161-97bc-95189fa1363d";
        const sse = new EventSource(`${BASE_URL}/realtime?user_id=${userId}&device_id=${crypto.randomUUID()}`);

        sse.onopen = () => {
            console.log('SSE connection opened');
            eventBus.emit('gateway-status', { online: true });
        };

        sse.onerror = (error) => {
            console.error('SSE error:', error);
            eventBus.emit('gateway-status', { online: false });
        };
        sse.addEventListener("message", (event) => {
            console.log("incoming message", event)
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-message', data);
            } catch (e) {
                console.error('Failed to parse SSE message', e);
            }
        })

        sse.addEventListener(":metadata", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-metadata', data);
            } catch (e) {
                console.error('Failed to parse SSE metadata', e);
            }
        })

        sse.addEventListener(":thought", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-thought', data);
            } catch (e) {
                console.error('Failed to parse SSE thought', e);
            }
        })

        sse.addEventListener(":tool_start", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-tool_start', data);
            } catch (e) {
                console.error('Failed to parse SSE tool_start', e);
            }
        })

        sse.addEventListener(":presence", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-presence', data);
            } catch (e) {
                console.error('Failed to parse SSE presence', e);
            }
        })

        sse.addEventListener(":pairing_success", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-pairing-success', data);
            } catch (e) {
                console.error('Failed to parse SSE pairing-success', e);
            }
        })

        sse.addEventListener(":evolution", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-evolution', data);
            } catch (e) {
                console.error('Failed to parse SSE evolution', e);
            }
        })


        return () => sse.close();
    },

    createConversation: (name: string) => {
        return apiFetch<Conversation>('/conversations', {
            method: 'POST',
            body: JSON.stringify({name})
        });
    },

    updateConversation: (id: string, name: string) => {
        return apiFetch<any>(`/conversations/${id}`, {
            method: 'PUT',
            body: JSON.stringify({name})
        });
    },

    deleteConversation: (id: string) => {
        return apiFetch<any>(`/conversations/${id}`, {
            method: 'DELETE'
        });
    },
    searchConversations: (query: string) => {
        return apiFetch<any[]>(`/conversations/search?q=${encodeURIComponent(query)}`);
    },
    getConversations: () => {
        return apiFetch<any[]>('/conversations', {
            method: "GET"
        });
    },
    getGraph: (conversationId?: string) => {
        const url = conversationId ? `/graph?conversation_id=${conversationId}` : '/graph';
        return apiFetch<any>(url);
    },
    searchGraph: (query: string) => {
        return apiFetch<any>(`/graph/search?q=${encodeURIComponent(query)}`);
    },
    getSoulHistory: (conversationId: string) => {
        return apiFetch<any[]>(`/conversations/${conversationId}/soul-history`);
    },
    restoreSoul: (conversationId: string, version: number) => {
        return apiFetch<any>(`/conversations/${conversationId}/restore-soul`, {
            method: 'POST',
            body: JSON.stringify({version})
        });
    },
    getPairingCode: (conversationId: string) => {
        return apiFetch<any>(`/conversations/${conversationId}/pairing`, {
            method: 'POST'
        });
    },
    getChannels: () => {
        return apiFetch<any>('/user/channels', {
            method: 'GET'
        });
    },
    getProfile: () => {
        return apiFetch<any>('/user/profile', {
            method: 'GET'
        });
    },
    logout: () => {
        return apiFetch<any>('/auth/logout', {
            method: 'POST'
        });
    },
    getWhatsappQr: () => {
        return fetch(`${CHANNEL_URL}/whatsapp/qr`).then(res => res.json());
    }
};
