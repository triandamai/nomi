import {chatApi} from '$lib/api/client';
import {conversationStore, type Conversation} from './conversation.svelte';
import {popupStore} from './popup.svelte';
import {eventBus} from '$lib/utils';

export function createSidebarStore() {
    let showUserMenu = $state(false);
    let reminders = $state<any[]>([]);
    let isLoadingReminders = $state(false);
    let hasMoreReminders = $state(true);
    let isPaired = $state(false);
    let pairingCode = $state('');
    let copied = $state(false);
    let newConvName = $state('');
    let newConvType = $state('private');
    let editingConv = $state<Conversation | null>(null);
    let channels = $state<any[]>([]);
    let whatsappQr = $state('');
    let currentPlatform = $state('');
    let isLoadingQr = $state(false);
    let isGatewayOnline = $state(false);
    let modelInfo = $state({
        agent_model: 'Loading...',
        rag_embedding: '...',
        media_classification: '...',
        media_analyze: '...'
    });

    return {
        get showUserMenu() {
            return showUserMenu;
        },
        set showUserMenu(value: boolean) {
            showUserMenu = value;
        },
        get reminders() {
            return reminders;
        },
        get isLoadingReminders() {
            return isLoadingReminders;
        },
        get hasMoreReminders() {
            return hasMoreReminders;
        },
        get isPaired() {
            return isPaired;
        },
        get pairingCode() {
            return pairingCode;
        },
        get copied() {
            return copied;
        },
        get newConvName() {
            return newConvName;
        },
        set newConvName(value: string) {
            newConvName = value;
        },
        get newConvType() {
            return newConvType;
        },
        set newConvType(value: string) {
            newConvType = value;
        },
        get editingConv() {
            return editingConv;
        },
        get channels() {
            return channels;
        },
        get whatsappQr() {
            return whatsappQr;
        },
        get currentPlatform() {
            return currentPlatform;
        },
        get isLoadingQr() {
            return isLoadingQr;
        },
        get isGatewayOnline() {
            return isGatewayOnline;
        },
        get modelInfo() {
            return modelInfo;
        },

        init() {
            this.checkPairingStatus();
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
            eventBus.subscribe('sse-pairing-success', (data: any) => {
                if (data.conversation_id === conversationStore.activeConversationId) {
                    isPaired = true;
                    this.checkPairingStatus();
                    popupStore.closeLast();
                }
            });
        },

        toggleUserMenu() {
            showUserMenu = !showUserMenu;
            if (showUserMenu) {
                this.fetchReminders();
                this.checkPairingStatus();
            }
        },

        async fetchReminders() {
            isLoadingReminders = true;
            try {
                const response = await chatApi.getReminders();
                if (response.data) {
                    reminders = response.data;
                    hasMoreReminders = response.data.length === 20;
                }
            } catch (e) {
                console.error('Failed to fetch reminders', e);
            } finally {
                isLoadingReminders = false;
            }
        },

        async loadMoreReminders() {
            if (isLoadingReminders || !hasMoreReminders) return;
            isLoadingReminders = true;
            try {
                const lastReminder = reminders[reminders.length - 1];
                const cursor = lastReminder ? lastReminder.due_at : null;
                const response = await chatApi.getReminders(cursor);
                if (response.data && response.data.length > 0) {
                    reminders = [...reminders, ...response.data];
                    hasMoreReminders = response.data.length === 20;
                } else {
                    hasMoreReminders = false;
                }
            } catch (e) {
                console.error('Failed to load more reminders', e);
            } finally {
                isLoadingReminders = false;
            }
        },

        async checkPairingStatus() {
            try {
                const data = await conversationStore.getChannels();
                if (data.data) {
                    channels = data.data.channels;
                    if (data.data.channels)
                        isPaired = data.data.channels.some((c: any) => c.paired);
                }
            } catch (e) {
                console.error('Failed to check pairing status', e);
            }
        },

        copyToClipboard() {
            navigator.clipboard.writeText(pairingCode);
            copied = true;
            setTimeout(() => copied = false, 2000);
        },

        setEditingConv(conv: Conversation | null) {
            editingConv = conv;
            if (conv) {
                newConvName = conv.name;
            } else {
                newConvName = '';
            }
        },

        async getPairingCode(id: string) {
            try {
                const data = await conversationStore.getPairingCode(id);
                pairingCode = data.pairing_code;
                return data;
            } catch (e) {
                console.error(e);
                throw e;
            }
        },

        async handlePairing(platform: string, pairingContent: any, pairingFooter: any) {
            if (!conversationStore.activeConversationId) return;
            currentPlatform = platform;
            try {
                const data = await conversationStore.getPairingCode(conversationStore.activeConversationId);
                pairingCode = data.pairing_code;
                popupStore.open({
                    title: `Link ${platform.charAt(0).toUpperCase() + platform.slice(1)}`,
                    width: 'max-w-md',
                    contentSnippet: pairingContent,
                    footerSnippet: pairingFooter
                });
            } catch (e) {
                console.error(e);
            }
        },

        async fetchWhatsappQr() {
            isLoadingQr = true;
            try {
                const res = await chatApi.getWhatsappQr();
                if (res && res.qr) {
                    whatsappQr = res.qr;
                }
            } catch (e) {
                console.error('Failed to fetch WhatsApp QR', e);
            } finally {
                isLoadingQr = false;
            }
        },

        async handleWhatsappLogout() {
            isLoadingQr = true;
            try {
                await chatApi.logoutWhatsapp();
                setTimeout(() => {
                    this.fetchWhatsappQr();
                }, 1000);
            } catch (e) {
                console.error('Failed to logout from WhatsApp', e);
            } finally {
                isLoadingQr = false;
            }
        },

        openWhatsappBotManager(whatsappBotSetupSnippet: any, pairingFooter: any) {
            currentPlatform = 'whatsapp';
            popupStore.open({
                title: 'WhatsApp Bot Setup',
                width: 'max-w-md',
                contentSnippet: whatsappBotSetupSnippet,
                footerSnippet: pairingFooter
            });
            this.fetchWhatsappQr();
        }
    };

}

export const sidebarStore = createSidebarStore();
