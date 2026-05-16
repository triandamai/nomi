import {eventBus} from '$lib/utils';
import type {Conversation} from "$lib/stores/conversation.svelte";
import {env} from '$env/dynamic/public';
import {getSession} from "$lib/stores/profile.svelte";

const BASE_URL = env.PUBLIC_GATEWAY_URL || 'http://localhost:8000/api';
const CHANNEL_URL = env.PUBLIC_CHANNEL_URL || 'http://localhost:8001/api';

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

    if (typeof window === 'undefined') {
        return {
            meta: {
                code: 500,
                message: "cannot get session"
            },
            data: null
        } as ApiResponse<T>
    }
    const [token] = getSession()
    const response = await fetch(`${BASE_URL}${endpoint}`, {
        ...options,
        headers: {
            'Content-Type': 'application/json',
            ...(token ? {'Authorization': `Bearer ${token}`} : {}),
            ...options.headers
        }
    });

    if (!response.ok) {
        const error = await response.json().catch(() => ({message: 'An unknown error occurred'}));
        if (error.meta?.message && error.meta.message.toLowerCase().includes('token limit reached')) {
            eventBus.emit('token-limit-reached', error.meta.message);
        }
        throw new Error(error.message || error.meta?.message || response.statusText);
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
        })
    },

    streamChat: async (message: string, conversationId: string, media?: {
        image_url?: string,
        audio_url?: string,
        video_url?: string,
        doc_url?: string
    }) => {
        const token = typeof window !== 'undefined' ? sessionStorage.getItem('auth_token') : null;
        const response = await fetch(`${BASE_URL}/chat/stream`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                ...(token ? {'Authorization': `Bearer ${token}`} : {}),
            },
            body: JSON.stringify({
                message,
                conversation_id: conversationId,
                ...media
            })
        });
        return response;
    },

    uploadFile: async (file: File) => {
        const token = typeof window !== 'undefined' ? sessionStorage.getItem('auth_token') : null;
        const formData = new FormData();
        formData.append('file', file);

        const response = await fetch(`${BASE_URL}/upload`, {
            method: 'POST',
            headers: {
                ...(token ? {'Authorization': `Bearer ${token}`} : {}),
            },
            body: formData
        });

        if (!response.ok) {
            throw new Error('Upload failed');
        }

        return response.json();
    },

    streamEvent() {
        const userId = typeof window !== 'undefined' ? sessionStorage.getItem('user_id') : null;
        const sse = new EventSource(`${BASE_URL}/realtime?user_id=${userId}&device_id=${crypto.randomUUID()}`);

        sse.onopen = () => {
            console.log('SSE connection opened');
            eventBus.emit('gateway-status', {online: true});
        };

        sse.onerror = (error) => {
            console.error('SSE error:', error);
            eventBus.emit('gateway-status', {online: false});
        };

        sse.addEventListener("message", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-message', data);
            } catch (e) {
                console.error('Failed to parse SSE message', e);
            }
        })

        sse.addEventListener("metadata", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-metadata', data);
            } catch (e) {
                console.error('Failed to parse SSE metadata', e);
            }
        })

        sse.addEventListener("thought", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-thought', data);
            } catch (e) {
                console.error('Failed to parse SSE thought', e);
            }
        })

        sse.addEventListener("tool_start", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-tool_start', data);
            } catch (e) {
                console.error('Failed to parse SSE tool_start', e);
            }
        })

        sse.addEventListener("tool_end", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-tool_end', data);
            } catch (e) {
                console.error('Failed to parse SSE tool_end', e);
            }
        })

        sse.addEventListener("token_update", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-token_update', data);
            } catch (e) {
                console.error('Failed to parse SSE token_update', e);
            }
        })

        sse.addEventListener("presence", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-presence', data);
            } catch (e) {
                console.error('Failed to parse SSE presence', e);
            }
        })

        sse.addEventListener("pairing_success", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-pairing-success', data);
            } catch (e) {
                console.error('Failed to parse SSE pairing-success', e);
            }
        })

        sse.addEventListener("evolution", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('sse-evolution', data);
            } catch (e) {
                console.error('Failed to parse SSE evolution', e);
            }
        })

        sse.addEventListener("stock_signal", (event) => {
            try {
                const data = JSON.parse(event.data);
                eventBus.emit('stock-signal', data);
            } catch (e) {
                console.error('Failed to parse SSE stock_signal', e);
            }
        })


        return () => sse.close();
    },

    createConversation: (name: string, type: string = 'private') => {
        return apiFetch<Conversation>('/conversations', {
            method: 'POST',
            body: JSON.stringify({name, conversation_type: type})
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
    getReminders: (cursor?: string, limit: number = 20) => {
        const url = new URL(`${BASE_URL}/reminders`);
        if (cursor) url.searchParams.append('cursor', cursor);
        url.searchParams.append('limit', limit.toString());
        return apiFetch<any[]>(url.pathname.replace("/api", "") + url.search);
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
    exploreStorage: (prefix?: string) => {
        const url = new URL(`${BASE_URL}/v1/admin/storage/explore`);
        if (prefix) url.searchParams.append('prefix', prefix);
        return apiFetch<any>(url.pathname.replace("/api", "") + url.search);
    },
    deleteStorage: (path: string) => {
        const url = new URL(`${BASE_URL}/v1/admin/storage/delete`);
        url.searchParams.append('path', path);
        return apiFetch<any>(url.pathname.replace("/api", "") + url.search, {
            method: 'DELETE'
        });
    },
    uploadToStorage: async (file: File, prefix?: string) => {
        const token = typeof window !== 'undefined' ? sessionStorage.getItem('auth_token') : null;
        const formData = new FormData();
        formData.append('file', file);

        const url = new URL(`${BASE_URL}/v1/admin/storage/upload`);
        if (prefix) url.searchParams.append('prefix', prefix);

        const response = await fetch(url.toString(), {
            method: 'POST',
            headers: {
                ...(token ? {'Authorization': `Bearer ${token}`} : {}),
            },
            body: formData
        });

        if (!response.ok) {
            throw new Error('Upload failed');
        }

        return response.json();
    },
    getMoneyHistory: (page: number = 1, query?: string, category?: string) => {
        const url = new URL(`${BASE_URL}/v1/money/history`);
        url.searchParams.append('page', page.toString());
        if (query) url.searchParams.append('query', query);
        if (category) url.searchParams.append('category', category);
        return apiFetch<any>(url.pathname.replace("/api", "") + url.search);
    },
    getHealthHistory: (startDate?: string, endDate?: string) => {
        const url = new URL(`${BASE_URL}/health/history`);
        if (startDate) url.searchParams.append('start_date', startDate);
        if (endDate) url.searchParams.append('end_date', endDate);
        return apiFetch<any>(url.pathname.replace("/api", "") + url.search);
    },
    updateMoneyHistory: (id: string, updates: { amount?: number, merchant_name?: string, category?: string }) => {
        return apiFetch<any>(`/v1/money/history/${id}`, {
            method: 'PATCH',
            body: JSON.stringify(updates)
        });
    },
    deleteMoneyHistory: (id: string) => {
        return apiFetch<any>(`/v1/money/history/${id}`, {
            method: 'DELETE'
        });
    },
    getAdminConversations: (cursor?: string, limit: number = 20) => {
        const url = new URL(`${BASE_URL}/v1/admin/conversations`);
        if (cursor) url.searchParams.append('cursor', cursor);
        url.searchParams.append('limit', limit.toString());
        return apiFetch<any>(url.pathname.replace("/api", "") + url.search);
    },
    updateAdminConversation: (id: string, updates: { max_token_usage?: number, title?: string }) => {
        return apiFetch<any>(`/v1/admin/conversations/${id}`, {
            method: 'PATCH',
            body: JSON.stringify(updates)
        });
    },
    getAdminUsers: (cursor?: string, limit: number = 20, query?: string) => {
        const url = new URL(`${BASE_URL}/v1/admin/users`);
        if (cursor) url.searchParams.append('cursor', cursor);
        url.searchParams.append('limit', limit.toString());
        if (query) url.searchParams.append('query', query);
        return apiFetch<any>(url.pathname.replace("/api", "") + url.search);
    },
    getAdminUserDetail: (id: string) => {
        return apiFetch<any>(`/v1/admin/users/${id}`);
    },
    updateAdminUser: (id: string, updates: {
        display_name?: string,
        name?: string,
        email?: string,
        role?: string,
        is_verified?: boolean
    }) => {
        return apiFetch<any>(`/v1/admin/users/${id}`, {
            method: 'PATCH',
            body: JSON.stringify(updates)
        });
    },
    deleteAdminUser: (id: string) => {
        return apiFetch<any>(`/v1/admin/users/${id}`, {
            method: 'DELETE'
        });
    },
    publishAdminInbound: (payload: any) => {
        return apiFetch<any>(`/v1/admin/redis/publish/inbound`, {
            method: 'POST',
            body: JSON.stringify(payload)
        });
    },
    publishAdminOutbound: (payload: any) => {
        return apiFetch<any>(`/v1/admin/redis/publish/outbound`, {
            method: 'POST',
            body: JSON.stringify(payload)
        });
    },
    logout: () => {
        return apiFetch<any>('/auth/logout', {
            method: 'POST'
        });
    },
    getWhatsappQr: () => {
        return fetch(`${CHANNEL_URL}/whatsapp/qr`).then(res => res.json());
    },
    logoutWhatsapp: () => {
        return fetch(`${CHANNEL_URL}/whatsapp/logout`, {
            method: 'POST'
        }).then(res => res.json());
    },
    joinWaitlist: (email: string) => {
        return apiFetch<any>('/waitlist', {
            method: 'POST',
            body: JSON.stringify({email})
        });
    }
};
