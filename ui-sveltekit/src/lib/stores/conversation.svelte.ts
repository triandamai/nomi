import { useAvatar } from '$lib/utils';
import { chatApi } from '$lib/api/client';
import {chatStore} from "$lib/stores/chat.svelte";

export type Conversation = {
    id: string;
    name: string;
    avatar?: string;
    active?: boolean;
    online?: boolean;
};

function createConversationStore() {
    let conversations = $state<Conversation[]>([]);
    
    let activeConversationId = $state<string>('');

    return {
        get conversations() {
            return conversations;
        },
        get activeConversationId() {
            return activeConversationId;
        },
        async loadConversations() {
            try {
                const response = await chatApi.getConversations();

                // Map API response to our Conversation type
                const loaded = response.data.map((c: any) => ({
                    id: c.id,
                    name: c.name || c.title || 'Untitled',
                    active: c.id === activeConversationId,
                    online: true
                }));
                conversations = loaded;
                
                if (conversations.length > 0 && !conversations.find(c => c.id === activeConversationId)) {
                    this.setActive(conversations[0].id);
                }
            } catch (error) {
                console.error('Failed to load conversations', error);
            }
        },
        setActive(id: string) {
            activeConversationId = id;
            conversations = conversations.map(c => ({
                ...c,
                active: c.id === id
            }));
            chatStore.fetchMessages(false).finally()
        },
        async addConversation(name: string) {
            try {
                const response = await chatApi.createConversation(name);
                // For now, if response is dummy, we generate a local one
                const newConv: Conversation = {
                    id: response.id || crypto.randomUUID(),
                    name: response.name || name,
                    active: false,
                    online: true
                };
                conversations = [...conversations, newConv];
                return newConv;
            } catch (error) {
                console.error('Failed to create conversation', error);
                // Fallback for demo/dummy purposes if API fails
                const newConv: Conversation = {
                    id: crypto.randomUUID(),
                    name,
                    active: false,
                    online: true
                };
                conversations = [...conversations, newConv];
                return newConv;
            }
        },
        async updateConversation(id: string, name: string) {
            try {
                await chatApi.updateConversation(id, name);
                conversations = conversations.map(c => 
                    c.id === id ? { ...c, name } : c
                );
            } catch (error) {
                console.error('Failed to update conversation', error);
                // Fallback
                conversations = conversations.map(c => 
                    c.id === id ? { ...c, name } : c
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
        }
    };
}

export const conversationStore = createConversationStore();
