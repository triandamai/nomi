import {chatApi} from '$lib/api/client';
import {eventBus} from '$lib/utils';
import {conversationStore} from "$lib/stores/conversation.svelte";

export type Message = {
    role: 'user' | 'assistant' | 'system';
    content: string;
    thought?: string;
    id: string;
    toolCalls?: Array<{ tool: any, result?: string }>;
};

function createChatStore() {
    let messages = $state<Message[]>([]);
    let loading = $state(false);
    let error = $state<string | null>(null);
    let conversationId = $state<string | undefined>(undefined);
    let nextCursor = $state<string | null>(null);
    let hasMore = $state(true);
    let isTyping = $state(false);
    let currentThought = $state<string>("");

    // Subscribe to SSE events via EventBus
    eventBus.subscribe('sse-message', (data) => {
        if (data.content) {
            messages = [...messages, {
                id: data.id || crypto.randomUUID(),
                role: "assistant", 
                content: data.content,
                thought: data.thought
            } as Message];
            currentThought = ""; // Clear thought when message arrives
            isTyping = false;
        }
    });

    eventBus.subscribe('sse-thought', (data) => {
        if (data.thought) {
            currentThought = data.thought;
        }
    });

    eventBus.subscribe('sse-presence', (data) => {
        if (data.user_id === 'nomi') {
            isTyping = data.is_typing;
        }
    });

    return {
        get messages() {
            return messages;
        },
        get loading() {
            return loading;
        },
        get error() {
            return error;
        },
        get conversationId() {
            return conversationId;
        },
        get hasMore() {
            return hasMore;
        },
        get currentThought() {
            return currentThought;
        },
        get isTyping() {
            return isTyping;
        },

        async fetchMessages(loadMore = false) {
            conversationId = conversationStore.activeConversationId
            if (!conversationId || (loadMore && !hasMore)) return;

            loading = true;
            error = null;
            try {
                const cursor = loadMore ? nextCursor : undefined;
                const response = await chatApi.getMessages(conversationId, cursor || undefined);
                
                // response structure is { data: { messages: [], next_cursor: string } }
                const newMessages = response.data?.messages || [];
                const nextC = response.data?.next_cursor || null;

                if (loadMore) {
                    // Prepend older messages
                    messages = [...newMessages.reverse(), ...messages];
                } else {
                    // Initial load or refresh
                    messages = newMessages.reverse();
                }

                nextCursor = nextC;
                hasMore = !!nextC;
            } catch (err) {
                error = err instanceof Error ? err.message : 'Failed to fetch messages';
                console.error('Fetch Messages Error:', err);
            } finally {
                loading = false;
            }
        },


        async sendMessage(content: string) {
            conversationId = conversationStore.activeConversationId
            if (!conversationId) {
                error = 'Conversation ID is missing';
                return;
            }

            const userMsg: Message = {
                id: crypto.randomUUID(),
                role: 'user',
                content
            };

            messages = [...messages, userMsg];
            loading = true;
            error = null;

            try {
                await chatApi.streamChat(content, conversationId);
            } catch (err) {
                error = err instanceof Error ? err.message : 'Failed to send message';
                console.error('Chat Store Error:', err);
            } finally {
                loading = false;
            }
        },

        clearMessages() {
            messages = [];
            conversationId = undefined;
        }
    };
}

export const chatStore = createChatStore();
