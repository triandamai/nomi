import {type ApiResponse, chatApi} from '$lib/api/client';
import {eventBus} from '$lib/utils';
import {mqttClient} from '$lib/api/mqtt';

export type Conversation = {
    id: string;
    name: string;
    avatar?: string;
    cumulative_tokens: number,
    active?: boolean;
    online?: boolean;
};

export const CONV_ID = "296e7dc6"

export function persistConversationId(value: string) {
    localStorage.setItem(CONV_ID, btoa(value))
}

export function getPersistConversationId(): string | null {
    const v = localStorage.getItem(CONV_ID)
    if (!v) return null
    return atob(v)
}

function createConversationStore() {
    let conversations = $state<Conversation[]>([]);

    let activeConversationId = $state<string>('');
    let activeConversation = $state<Conversation | null>(null)

    // Subscribe to token updates
    eventBus.subscribe('sse-token_update', (data) => {
        if (data.conversation_id) {
            conversations = conversations.map(c => {
                if (c.id === data.conversation_id) {
                    return {...c, cumulative_tokens: data.cumulative_tokens};
                }
                return c;
            });

            if (activeConversationId === data.conversation_id && activeConversation) {
                activeConversation.cumulative_tokens = data.cumulative_tokens;
            }
        }
    });

    return {
        get conversations() {
            return conversations;
        },
        get activeConversationId() {
            return activeConversationId;
        },
        get activeConversation(): Conversation | null {
            return activeConversation
        },
        async loadConversations() {
            try {
                const response = await chatApi.getConversations();

                if(response.data) {
                    // Map API response to our Conversation type
                    const loaded = response.data.map((c: any) => ({
                        id: c.id,
                        name: c.name || c.title || 'Untitled',
                        active: c.id === activeConversationId,
                        cumulative_tokens: c.cumulative_tokens,
                        online: true
                    }));
                    conversations = loaded;

                    const currentId = getPersistConversationId()
                    if (currentId != null) {
                        const find = conversations.find(c => c.id === currentId)
                        if (conversations.length > 0 && find) {
                            this.setActive(find.id);
                        }
                    }
                }
            } catch (error) {
                console.error('Failed to load conversations', error);
            }
        },
        setActive(id: string) {
            persistConversationId(id)
            activeConversationId = id;
            const find = conversations.find(c => c.id == id)
            activeConversation = find || null
            conversations = conversations.map(c => ({
                ...c,
                active: c.id === id
            }));
            mqttClient.setConversation(id);
        },
        async addConversation(name: string, type: string = 'private') {
            try {
                const response = await chatApi.createConversation(name, type);
                if (response.data) {
                    // For now, if response is dummy, we generate a local one
                    const newConv: Conversation = {
                        id: response.data.id || crypto.randomUUID(),
                        name: response.data.name || name,
                        cumulative_tokens: response.data.cumulative_tokens,
                        active: false,
                        online: true
                    };
                    conversations = [...conversations, newConv];
                    return newConv;
                }
            } catch (error) {
                console.error('Failed to create conversation', error);
            }
        },
        async updateConversation(id: string, name: string) {
            try {
                await chatApi.updateConversation(id, name);
                conversations = conversations.map(c =>
                    c.id === id ? {...c, name} : c
                );
            } catch (error) {
                console.error('Failed to update conversation', error);
                // Fallback
                conversations = conversations.map(c =>
                    c.id === id ? {...c, name} : c
                );
            }
        },
        async deleteConversation(id: string) {
            try {
                await chatApi.deleteConversation(id);
                conversations = conversations.filter(c => c.id !== id);
                if (activeConversationId === id && conversations.length > 0) {
                    this.setActive(conversations[0].id);
                }
            } catch (error) {
                console.error('Failed to delete conversation', error);
                // Fallback
                conversations = conversations.filter(c => c.id !== id);
                if (activeConversationId === id && conversations.length > 0) {
                    this.setActive(conversations[0].id);
                }
            }
        },
        async getPairingCode(id: string) {
            try {
                const response = await chatApi.getPairingCode(id);
                return response.data;
            } catch (error) {
                console.error('Failed to get pairing code', error);
                throw error;
            }
        },
        async getChannels():Promise<ApiResponse<any>> {
            try {
                const response = await chatApi.getChannels();
                return response;
            } catch (error) {
                console.error('Failed to get user channels', error);
                return {
                    meta:{
                        code:500,
                        message:(error as any).message
                    },
                    data:null
                } as ApiResponse<any>
            }
        }
    };
}

export const conversationStore = createConversationStore();
