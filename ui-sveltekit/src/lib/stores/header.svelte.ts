import { chatApi } from '$lib/api/client';
import { conversationStore } from './conversation.svelte';
import { profileStore } from './profile.svelte';
import { popupStore } from './popup.svelte';
import { eventBus } from '$lib/utils';

function createHeaderStore() {
    let isGatewayOnline = $state(false);

    return {
        get isGatewayOnline() { return isGatewayOnline; },
        get modelInfo() { 
            return profileStore.modelInfo || {
                agent_model: 'Loading...',
                rag_embedding: '...',
                media_classification: '...',
                media_analyze: '...'
            }; 
        },

        init() {
            eventBus.subscribe('gateway-status', (data) => {
                isGatewayOnline = data.online;
            });
        }
    };
}

export const headerStore = createHeaderStore();
