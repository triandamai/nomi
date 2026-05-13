import { chatApi } from '$lib/api/client';
import { conversationStore } from './conversation.svelte';
import { popupStore } from './popup.svelte';
import { eventBus } from '$lib/utils';

function createHeaderStore() {
    let isGatewayOnline = $state(false);
    let modelInfo = $state({
        agent_model: 'Loading...',
        rag_embedding: '...',
        media_classification: '...',
        media_analyze: '...'
    });

    return {
        get isGatewayOnline() { return isGatewayOnline; },
        get modelInfo() { return modelInfo; },

        init() {
            eventBus.subscribe('gateway-status', (data) => {
                isGatewayOnline = data.online;
            });

            eventBus.subscribe('sse-metadata', (data) => {
                modelInfo = {
                    agent_model: data.agent_model || 'Unknown',
                    rag_embedding: data.rag_embedding || 'Unknown',
                    media_classification: data.media_classification || 'Unknown',
                    media_analyze: data.media_analyze || 'Unknown'
                };
            });
        }
    };
}

export const headerStore = createHeaderStore();
