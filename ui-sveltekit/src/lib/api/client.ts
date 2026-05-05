import { eventBus } from '$lib/utils';

const BASE_URL = 'http://localhost:8000/api';
const CHANNEL_URL = 'http://localhost:3001/api';

export async function apiFetch<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
	const response = await fetch(`${BASE_URL}${endpoint}`, {
		...options,
		headers: {
			'Content-Type': 'application/json',
			...options.headers
		}
	});

	if (!response.ok) {
		const error = await response.json().catch(() => ({ message: 'An unknown error occurred' }));
		throw new Error(error.message || response.statusText);
	}

	return response.json();
}

export const chatApi = {
	sendMessage: (message: string, conversationId?: string) => {
		return apiFetch<{ reply: string }>('/chat', {
			method: 'POST',
			body: JSON.stringify({ message, conversation_id: conversationId })
		});
	},

	getMessages: (conversationId: string, cursor?: string, limit: number = 20) => {
		const url = new URL(`${BASE_URL}/conversations/${conversationId}/messages`);
		if (cursor) url.searchParams.append('cursor', cursor);
		url.searchParams.append('limit', limit.toString());
		return apiFetch<any>(url.pathname.replace("/api","") + url.search);
	},

	streamChat: async (message: string, conversationId: string) => {
		const response = await fetch(`${BASE_URL}/chat/stream`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ message, conversation_id: conversationId })
		});
		return response;
	},

	streamEvent() {
		// Based on gateway-rust/src/routes.rs, the SSE endpoint is /realtime
		const sse = new EventSource(`${BASE_URL}/realtime?user_id=9220f30e-b5cb-4161-97bc-95189fa1363d&device_id=${crypto.randomUUID()}`);
		
		sse.onopen = () => {
			console.log('SSE connection opened');
		};

		sse.onerror = (error) => {
			console.error('SSE error:', error);
		};
		sse.addEventListener("message",(event)=>{
			console.log("incoming message",event)
			try {
				const data = JSON.parse(event.data);
				eventBus.emit('sse-message', data);
			} catch (e) {
				console.error('Failed to parse SSE message', e);
			}
		})

		sse.addEventListener(":thought",(event)=>{
			try {
				const data = JSON.parse(event.data);
				eventBus.emit('sse-thought', data);
			} catch (e) {
				console.error('Failed to parse SSE thought', e);
			}
		})

		sse.addEventListener(":tool_start",(event)=>{
			try {
				const data = JSON.parse(event.data);
				eventBus.emit('sse-tool_start', data);
			} catch (e) {
				console.error('Failed to parse SSE tool_start', e);
			}
		})

		sse.addEventListener(":presence",(event)=>{
			try {
				const data = JSON.parse(event.data);
				eventBus.emit('sse-presence', data);
			} catch (e) {
				console.error('Failed to parse SSE presence', e);
			}
		})


		return () => sse.close();
	},

	createConversation: (name: string) => {
		return apiFetch<any>('/conversations', {
			method: 'POST',
			body: JSON.stringify({ name })
		});
	},

	updateConversation: (id: string, name: string) => {
		return apiFetch<any>(`/conversations/${id}`, {
			method: 'PUT',
			body: JSON.stringify({ name })
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
		return apiFetch<any[]>('/conversations',{
			method:"GET"
		});
	},
	getGraph: () => {
		return apiFetch<any>('/graph');
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
			body: JSON.stringify({ version })
		});
	},
	getWhatsappQr: () => {
		return fetch(`${CHANNEL_URL}/whatsapp/qr`).then(res => res.json());
	}
};
