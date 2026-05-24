import {type ApiResponse, chatApi} from '$lib/api/client';
import {eventBus} from '$lib/utils';
import {conversationStore, getPersistConversationId} from "$lib/stores/conversation.svelte";
import toast from 'svelte-french-toast';

export type Message = {
    role: 'user' | 'assistant' | 'system';
    content: string;
    display_name:string|null,
    thought?: string;
    image_url?: string;
    video_url?: string;
    audio_url?: string;
    document_url?: string;
    sticker_url?: string;
    id: string;
    user_id?: string;
    total_tokens: number;
    created_at?: string;
    metadata?: Record<string, any>;
    reply_to_id?: string;
    replied_message?: {
        id: string;
        role: string;
        content: string;
        display_name: string | null;
    };
    toolCalls?: Array<{ tool: any, result?: string }>;
};

function createChatStore() {
    let messages = $state<Message[]>([]);
    let loading = $state(false);
    let error = $state<string | null>(null);
    let conversationId = $state<string | undefined>(undefined);
    let nextCursor = $state<string | null>(null);
    let hasMore = $state(true);
    let replyingToMessage = $state<Message | null>(null);
    
    // Scoped state maps
    let thoughts = $state<Record<string, string>>({});
    let isTyping = $state<Record<string, boolean>>({});
    let activeTools = $state<Record<string, string[]>>({});

    // Subscribe to SSE events via EventBus
    eventBus.subscribe('sse-message', (data) => {
        const cid = data.conversation_id;
        
        // Only process message if it belongs to the current active conversation
        if (cid && cid !== conversationStore.activeConversationId) {
            // Show alert/notification for message from another conversation
            toast.success(`New message in another conversation: ${data.content.substring(0, 50)}${data.content.length > 50 ? '...' : ''}`, {
                duration: 5000,
                position: 'top-right',
                style: 'background: #1e1e1e; color: #fff; border: 1px solid #333;'
            });
            return;
        }

        if (data.id) {
            const find = messages.findIndex(v => v.id == data.id)
            if (find >= 0) {
                messages[find] = ({
                    id: data.id || crypto.randomUUID(),
                    role: data.role || "assistant",
                    content: data.content,
                    thought: data.thought,
                    image_url: data.image_url,
                    user_id: data.user_id,
                    total_tokens: data.total_tokens,
                    metadata: data.metadata,
                    reply_to_id: data.reply_to_id,
                    replied_message: data.replied_message,
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
                    metadata: data.metadata,
                    reply_to_id: data.reply_to_id,
                    replied_message: data.replied_message,
                    created_at: data.created_at
                } as Message)

            }

            if (cid) {
                thoughts[cid] = ""; // Clear thought when message arrives
                activeTools[cid] = []; // Clear tools when message arrives
                isTyping[cid] = false;
            }
        }
    });

    eventBus.subscribe('sse-thought', (data) => {
        const cid = data.conversation_id;
        if (!cid) return;

        if (thoughts[cid] === undefined) thoughts[cid] = "";

        if (data.thought) {
            thoughts[cid] += data.thought;
        } else if (data.text) {
            // Status updates use 'text' field
            thoughts[cid] = data.text;
        }
    });

    eventBus.subscribe('sse-tool_start', (data) => {
        const cid = data.conversation_id;
        if (!cid || !data.name) return;

        if (!activeTools[cid]) activeTools[cid] = [];
        activeTools[cid].push(data.name);
    });

    eventBus.subscribe('sse-tool_end', (data) => {
        const cid = data.conversation_id;
        if (!cid || !data.name) return;

        if (activeTools[cid]) {
            activeTools[cid] = activeTools[cid].filter(tool => tool !== data.name);
        }
    });

    eventBus.subscribe('sse-presence', (data) => {
        const cid = data.conversation_id;
        if (!cid) return;

        if (data.user_id === 'nomi' || data.user_id === 'nomi-auth' || data.user_id === 'system' || data.user_id === 'assistant') {
            isTyping[cid] = data.is_typing;
        }
    });

    eventBus.subscribe('sse-evolution', (data) => {
        // Show notification to user
        toast.success(data.message || "Nomi has updated her core instructions to better suit your needs. ✨", {
            duration: 6000,
            position: 'bottom-right',
            style: 'background: #1e1e1e; color: #fff; border: 1px solid #333;'
        });
    });

    eventBus.subscribe('token-limit-reached', (message) => {
        toast.error(message || "Token limit reached for this conversation.", {
            duration: 5000,
            position: 'top-center',
            style: 'background: #7f1d1d; color: #fff; border: 1px solid #991b1b;'
        });
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
            const cid = conversationStore.activeConversationId;
            return cid ? (thoughts[cid] || "") : "";
        },
        get activeTool() {
            const cid = conversationStore.activeConversationId;
            const tools = cid ? (activeTools[cid] || []) : [];
            return tools.length > 0 ? tools[tools.length - 1] : null;
        },
        get isTyping() {
            const cid = conversationStore.activeConversationId;
            return cid ? (isTyping[cid] || false) : false;
        },
        get replyingToMessage() {
            return replyingToMessage;
        },
        set replyingToMessage(val: Message | null) {
            replyingToMessage = val;
        },

        async fetchMessages(loadMore = false) {
            const id = getPersistConversationId()
            if(!id) return
            conversationId = id
            if (!conversationId || (loadMore && !hasMore)) return;
            loading = true;
            error = null;
            try {
                const cursor = loadMore ? nextCursor : undefined;
                const response = await chatApi.getMessages(conversationId, cursor || undefined);

                // response structure is { data: { messages: [], next_cursor: string } }
                const data = response.data as any
                const newMessages = data?.messages || [];
                const nextC = data?.next_cursor || null;

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

            const reply_to_id = replyingToMessage?.id;
            replyingToMessage = null; // Clear after sending

            const response:any = await chatApi.streamChat(content, conversationId, { ...media, reply_to_id }).then((res:any)=>res.json()).catch(err => {
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
