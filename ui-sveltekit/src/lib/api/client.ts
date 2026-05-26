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
    lookupExternalUser: async (externalId: string) => {
        return apiFetch<{ user_id: string, display_name: string } | null>(`/users/lookup/${externalId}`);
    },

    searchUsers: async (query: string) => {
        return apiFetch<any>(`/users/search?q=${encodeURIComponent(query)}`);
    },

    sendMessage: async (message: string, conversationId: string, media: any = {}) => {
        if (typeof window === 'undefined') {
            return {
                meta: {
                    code: 500,
                    message: "cannot get session"
                },
                data: null
            } as ApiResponse<any>
        }
        const [token] = getSession()
        const response = await fetch(`${BASE_URL}/chat`, {
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

    streamChat: async (message: string, conversationId: string, media: any = {}) => {
        if (typeof window === 'undefined') {
            return {
                meta: {
                    code: 500,
                    message: "cannot get session"
                },
                data: null
            } as ApiResponse<any>
        }
        const [token] = getSession()
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
        if (typeof window === 'undefined') {
            return {
                meta: {
                    code: 500,
                    message: "cannot get session"
                },
                data: null
            } as ApiResponse<any>
        }
        const [token] = getSession()
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
    getMessages: (conversationId: string, cursor?: string, limit: number = 20) => {
        const url = new URL(`${BASE_URL}/conversations/${conversationId}/messages`);
        if (cursor) url.searchParams.append('cursor', cursor);
        url.searchParams.append('limit', limit.toString());
        return apiFetch<any[]>(`/conversations/${conversationId}/messages` + url.search);
    },
    getConversationMembers: (conversationId: string) => {
        return apiFetch<any>(`/conversations/${conversationId}/members`, {
            method: 'GET'
        });
    },
    getGraph: (conversationId?: string, month?: number, year?: number) => {
        const url = new URL(`${BASE_URL}/graph`);
        if (conversationId) url.searchParams.append('conversation_id', conversationId);
        if (month) url.searchParams.append('month', month.toString());
        if (year) url.searchParams.append('year', year.toString());
        return apiFetch<any>(`/graph` + url.search);
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
        return apiFetch<any[]>('/reminders' + url.search);
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
        return apiFetch<any>('/auth/profile', {
            method: 'GET'
        });
    },
    getModelInfo: () => {
        return apiFetch<any>('/model/info', {
            method: 'GET'
        });
    },
    updateProfile: (displayName: string) => {
        return apiFetch<any>('/auth/profile', {
            method: 'PUT',
            body: JSON.stringify({ display_name: displayName })
        });
    },
    exploreStorage: (prefix?: string) => {
        const url = new URL(`${BASE_URL}/v1/admin/storage/explore`);
        if (prefix) url.searchParams.append('prefix', prefix);
        return apiFetch<any>('/v1/admin/storage/explore' + url.search);
    },
    deleteStorage: (path: string) => {
        const url = new URL(`${BASE_URL}/v1/admin/storage/delete`);
        url.searchParams.append('path', path);
        return apiFetch<any>('/v1/admin/storage/delete' + url.search, {
            method: 'DELETE'
        });
    },
    uploadToStorage: async (file: File, prefix?: string) => {
        if (typeof window === 'undefined') {
            return {
                meta: {
                    code: 500,
                    message: "cannot get session"
                },
                data: null
            } as ApiResponse<any>
        }
        const [token] = getSession()
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
        return apiFetch<any>('/v1/money/history' + url.search);
    },
    getHealthHistory: (startDate?: string, endDate?: string) => {
        const url = new URL(`${BASE_URL}/health/history`);
        if (startDate) url.searchParams.append('start_date', startDate);
        if (endDate) url.searchParams.append('end_date', endDate);
        return apiFetch<any>('/health/history' + url.search);
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
        return apiFetch<any>('/v1/admin/conversations' + url.search);
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
        return apiFetch<any>('/v1/admin/users' + url.search);
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
    requestOtp: (externalId: string, channel: string = 'email') => {
        return apiFetch<any>('/auth/request-otp', {
            method: 'POST',
            body: JSON.stringify({identity: externalId, external_id: externalId, channel})
        });
    },
    verifyOtp: (externalId: string, code: string) => {
        return apiFetch<any>('/auth/verify-otp', {
            method: 'POST',
            body: JSON.stringify({identity: externalId, external_id: externalId, code})
        });
    },
    getAvailableTools: () => {
        return apiFetch<any[]>('/tools');
    },
    getGuardrailPatterns: () => {
        return apiFetch<any[]>('/v1/admin/guardrails/patterns');
    },
    insertGuardrailPattern: (content: string) => {
        return apiFetch<any>('/v1/admin/guardrails/patterns', {
            method: 'POST',
            body: JSON.stringify({content})
        });
    },
    deleteGuardrailPattern: (id: string) => {
        return apiFetch<any>(`/v1/admin/guardrails/patterns/${id}`, {
            method: 'DELETE'
        });
    },
    getSkillSchemas: () => {
        return apiFetch<any[]>('/skills/schemas');
    },
    executeSkill: (pluginName: string, args: any, conversationId?: string) => {
        return apiFetch<string>('/skills/execute', {
            method: 'POST',
            body: JSON.stringify({
                plugin_name: pluginName,
                args,
                conversation_id: conversationId
            })
        });
    },
    getDocumentation: () => {
        return apiFetch<string>('/readme');
    },
    getSkillsDocumentation: () => {
        return apiFetch<string>('/skills/readme');
    },
    getEdgeFunctions: () => {
        return apiFetch<any[]>('/plugins');
    },
    createEdgeFunction: (payload: any) => {
        return apiFetch<any>('/plugins', {
            method: 'POST',
            body: JSON.stringify(payload)
        });
    },
    updateEdgeFunction: (slug: string, payload: any) => {
        return apiFetch<any>(`/plugins/${slug}`, {
            method: 'PUT',
            body: JSON.stringify(payload)
        });
    },
    deleteEdgeFunction: (slug: string) => {
        return apiFetch<any>(`/plugins/${slug}`, {
            method: 'DELETE'
        });
    },
    executeEdgeFunction: (scriptCode: string, args: any) => {
        return apiFetch<{result: string, logs: string}>('/plugins/execute', {
            method: 'POST',
            body: JSON.stringify({
                script_code: scriptCode,
                args: args
            })
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
    },
    getPublicSkills: () => {
        return apiFetch<any[]>('/skills');
    },
    getSrpState: (slug: string) => {
        return apiFetch<any>(`/srp/${slug}`);
    },
    testSrp: (slug: string, text: string) => {
        return apiFetch<any>('/srp/test', {
            method: 'POST',
            body: JSON.stringify({slug, text})
        });
    },
    learnSrp: (slug: string, text: string) => {
        return apiFetch<any>('/srp/learn', {
            method: 'POST',
            body: JSON.stringify({slug, text})
        });
    },
    getAvailablePlugins: () => {
        return apiFetch<string[]>('/srp/available');
    },
    getProposals: () => {
        return apiFetch<any[]>('/srp/proposals');
    },
    getProposalLogs: (slug: string) => {
        return apiFetch<any>(`/srp/proposals/${slug}/logs`, {
            method: 'GET'
        });
    },
    updateProposal: (slug: string, payload: any) => {
        return apiFetch<any>(`/srp/proposals/${slug}`, {
            method: 'PUT',
            body: JSON.stringify(payload)
        });
    },
    deleteProposal: (slug: string) => {
        return apiFetch<any>(`/srp/proposals/${slug}`, {
            method: 'DELETE'
        });
    },
    approveProposal: (slug: string) => {
        return apiFetch<any>(`/srp/proposals/${slug}/approve`, {
            method: 'POST'
        });
    },
    deployProposal: (slug: string) => {
        return apiFetch<any>(`/srp/proposals/${slug}/deploy`, {
            method: 'POST'
        });
    }
};

export const api = {
    get: <T>(endpoint: string) => apiFetch<T>(endpoint, { method: 'GET' }),
    post: <T>(endpoint: string, body?: any) => apiFetch<T>(endpoint, { 
        method: 'POST', 
        body: body ? JSON.stringify(body) : undefined 
    }),
    patch: <T>(endpoint: string, body?: any) => apiFetch<T>(endpoint, { 
        method: 'PATCH', 
        body: body ? JSON.stringify(body) : undefined 
    }),
    put: <T>(endpoint: string, body?: any) => apiFetch<T>(endpoint, { 
        method: 'PUT', 
        body: body ? JSON.stringify(body) : undefined 
    }),
    delete: <T>(endpoint: string) => apiFetch<T>(endpoint, { method: 'DELETE' }),
};
