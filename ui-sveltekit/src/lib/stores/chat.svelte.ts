import {chatApi} from '$lib/api/client';
import {eventBus} from '$lib/utils';
import {conversationStore} from "$lib/stores/conversation.svelte";

export type Message = {
    role: 'user' | 'assistant' | 'system';
    content: string;
    thought?: string;
    image_url?: string;
    id: string;
    user_id?: string;
    total_tokens: number;
    created_at?: string;
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
    let activeTool = $state<string | null>(null);

    // Subscribe to SSE events via EventBus
    eventBus.subscribe('sse-message', (data) => {
        if (data.id) {
            const find = messages.findIndex(v => v.id == data.id)
            if (find > 0) {
                messages[find] = ({
                    id: data.id || crypto.randomUUID(),
                    role: data.role || "assistant",
                    content: data.content,
                    thought: data.thought,
                    image_url: data.image_url,
                    user_id: data.user_id,
                    total_tokens: data.total_tokens,
                    created_at: data.created_at
                } as Message)
            } else {
                messages.push({
                    id: data.id || crypto.randomUUID(),
                    role: data.role || "assistant",
                    content: data.content,
                    thought: data.thought,
                    image_url: data.image_url,
                    user_id: data.user_id,
                    total_tokens: data.total_tokens,
                    created_at: data.created_at
                } as Message)

            }

            currentThought = ""; // Clear thought when message arrives
            activeTool = null; // Clear tool when message arrives
            isTyping = false;
        }
    });

    eventBus.subscribe('sse-thought', (data) => {
        if (data.thought) {
            currentThought = data.thought;
        }
    });

    eventBus.subscribe('sse-tool_start', (data) => {
        if (data.name) {
            activeTool = data.name;
        }
    });

    eventBus.subscribe('sse-tool_end', (data) => {
        activeTool = null;
    });

    eventBus.subscribe('sse-presence', (data) => {
        if (data.user_id === 'nomi') {
            isTyping = data.is_typing;
        }
    });

    eventBus.subscribe('sse-evolution', (data) => {
        // Show notification to user
        alert(data.message || "Nomi has updated her core instructions to better suit your needs. ✨");
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
        get activeTool() {
            return activeTool;
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


        async sendMessage(content: string, media?: {
            image_url?: string,
            audio_url?: string,
            video_url?: string,
            doc_url?: string
        }): Promise<{
            isLimit: boolean,
            isSuccess: boolean
        }> {
            conversationId = conversationStore.activeConversationId
            if (!conversationId) {
                error = 'Conversation ID is missing';
                loading = true;
                return {
                    isSuccess: false,
                    isLimit: false
                };
            }

            loading = true;
            error = null;


            const response = await chatApi.streamChat(content, conversationId, media).then((res) => res.json()).catch(err => {
                error = err instanceof Error ? err.message : 'Failed to send message';
            }).finally(() => {
                loading = false;
            });
            return {
                isSuccess: response.meta.code >= 200 && response.meta.code <= 209,
                isLimit: response.meta.code === 1000
            }
        },

        clearMessages() {
            messages = [];
            conversationId = undefined;
        }
    };
}

export const chatStore = createChatStore();
