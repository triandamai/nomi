<script lang="ts">
    import {Check, ChevronRight, Copy, Cpu, Link, MessageSquare, Network, Settings2, RefreshCw} from 'lucide-svelte';
    import {page} from '$app/state';
    import SoulTimeline from './SoulTimeline.svelte';
    import QRCode from './QRCode.svelte';
    import {conversationStore} from '$lib/stores/conversation.svelte';
    import {popupStore} from '$lib/stores/popup.svelte';
    import {chatApi} from '$lib/api/client';
    import {eventBus} from '$lib/utils';
    import {onMount} from "svelte";

    let activeTab = $derived(page.url.pathname === '/rag' ? 'RAG' : 'Chat');
    let pairingCode = $state('');
    let whatsappQr = $state('');
    let currentPlatform = $state('');
    let copied = $state(false);
    let isLoadingQr = $state(false);

    // Task 2: Liveness Monitor
    let isGatewayOnline = $state(false);
    
    // Task 3: Dynamic Model Info
    let modelInfo = $state({ model: 'N/A', version: '' });

    // Task 1 & 4: Pairing Status
    let isPaired = $state(false);
    let channels = $state<any[]>([]);

    onMount(() => {
        checkPairingStatus();
    });

    async function checkPairingStatus() {
        try {
            const data = await conversationStore.getChannels();
            channels = data.channels;
            // Check if telegram or whatsapp is paired
            isPaired = data.channels.some((c: any) => c.paired);
        } catch (e) {
            console.error('Failed to check pairing status', e);
        }
    }

    eventBus.subscribe('gateway-status', (data) => {
        isGatewayOnline = data.online;
    });

    eventBus.subscribe('sse-metadata', (data) => {
        modelInfo = { model: data.model, version: data.version };
    });

    const tabs = [
        { name: 'Chat', href: '/', icon: MessageSquare },
        { name: 'RAG', href: '/rag', icon: Network }
    ];

    eventBus.subscribe('sse-pairing-success', (data: any) => {
        if (data.conversation_id === conversationStore.activeConversationId) {
            isPaired = true; // Task 4: Real-time feedback
            checkPairingStatus(); // Refresh channel list
            popupStore.closeLast();
        }
    });

    function openTimeline() {
        popupStore.open({
            title: 'Soul Timeline',
            width: 'w-1/3',
            contentSnippet: soulTimelineSnippet
        });
    }

    function openConnectionManager() {
        popupStore.open({
            title: 'App Connections',
            width: 'max-w-md',
            contentSnippet: connectionManagementSnippet
        });
    }

    function openWhatsappBotManager() {
        currentPlatform = 'whatsapp';
        popupStore.open({
            title: 'WhatsApp Bot Setup',
            width: 'max-w-md',
            contentSnippet: whatsappBotSetupSnippet,
            footerSnippet: pairingFooter
        });
        fetchWhatsappQr();
    }

    async function handlePairing(platform: string) {
        if (!conversationStore.activeConversationId) return;
        currentPlatform = platform;
        
        try {
            // Always fetch internal pairing code first
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
    }

    async function fetchWhatsappQr() {
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
    }

    function copyToClipboard() {
        navigator.clipboard.writeText(pairingCode);
        copied = true;
        setTimeout(() => copied = false, 2000);
    }
</script>

{#snippet soulTimelineSnippet()}
    {#if conversationStore.activeConversationId}
        <SoulTimeline conversationId={conversationStore.activeConversationId} />
    {/if}
{/snippet}

{#snippet connectionManagementSnippet()}
    <div class="space-y-4 py-2">
        <div class="flex items-center justify-between px-1">
            <p class="text-xs text-zinc-500 font-medium">Manage your connected messaging platforms.</p>
            <button 
                onclick={openWhatsappBotManager}
                class="text-[10px] font-black uppercase tracking-tighter text-emerald-500 hover:text-emerald-400 transition-colors bg-emerald-500/5 px-2 py-1 rounded border border-emerald-500/10"
            >
                Bot Setup
            </button>
        </div>
        <div class="grid gap-3">
            {#each channels as channel}
                <div class="flex items-center justify-between p-4 bg-zinc-950 border border-zinc-800 rounded-xl hover:border-zinc-700 transition-colors">
                    <div class="flex items-center gap-4">
                        <div class="w-10 h-10 rounded-full bg-zinc-900 border border-zinc-800 flex items-center justify-center text-zinc-400">
                            {#if channel.platform === 'telegram'}
                                <svg viewBox="0 0 24 24" class="w-5 h-5 fill-current"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.64 6.8c-.15 1.58-.8 5.42-1.13 7.19-.14.75-.42 1-.68 1.03-.58.05-1.02-.38-1.58-.75-.88-.58-1.38-.94-2.23-1.5-.99-.65-.35-1.01.22-1.59.15-.15 2.71-2.48 2.76-2.69a.2.2 0 00-.05-.18c-.06-.05-.14-.03-.21-.02-.09.02-1.49.95-4.22 2.79-.4.27-.76.41-1.08.4-.36-.01-1.04-.2-1.55-.37-.63-.2-1.12-.31-1.08-.66.02-.18.27-.36.74-.55 2.92-1.27 4.86-2.11 5.83-2.51 2.78-1.16 3.35-1.36 3.73-1.36.08 0 .27.02.39.12.1.08.13.19.14.27-.01.06.01.24 0 .38z"/></svg>
                            {:else if channel.platform === 'whatsapp'}
                                <svg viewBox="0 0 24 24" class="w-5 h-5 fill-current"><path d="M12.04 2c-5.46 0-9.91 4.45-9.91 9.91 0 1.75.46 3.45 1.32 4.95L2.05 22l5.25-1.38c1.45.79 3.08 1.21 4.74 1.21 5.46 0 9.91-4.45 9.91-9.91 0-2.65-1.03-5.14-2.9-7.01A9.817 9.817 0 0012.04 2m.01 1.67c2.2 0 4.26.86 5.82 2.42 1.56 1.56 2.41 3.63 2.41 5.83 0 4.54-3.7 8.23-8.24 8.23-1.48 0-2.93-.39-4.19-1.15l-.3-.17-3.12.82.83-3.04-.19-.3a8.132 8.132 0 01-1.26-4.38c.01-4.54 3.7-8.24 8.24-8.24m-3.53 4.75c-.19 0-.52.07-.79.37-.27.3-.87.85-.87 2.08s.89 2.42 1.01 2.58c.12.16 1.75 2.67 4.23 3.74.59.26 1.05.41 1.41.52.59.19 1.13.16 1.56.1.48-.07 1.47-.6 1.67-1.18.21-.58.21-1.07.14-1.18-.06-.1-.23-.16-.48-.27-.25-.12-1.47-.73-1.69-.82-.23-.09-.39-.12-.56.12-.17.25-.64.81-.78.97-.14.17-.29.19-.54.06-.25-.12-1.05-.39-1.99-1.23-.74-.66-1.23-1.47-1.38-1.72-.14-.25-.01-.39.11-.51.11-.11.25-.29.37-.43.12-.14.17-.25.25-.41.08-.16.04-.31-.02-.43-.06-.12-.56-1.35-.77-1.85-.2-.5-.4-.43-.56-.44l-.48-.01z"/></svg>
                            {:else}
                                <MessageSquare size={20} />
                            {/if}
                        </div>
                        <div>
                            <p class="text-sm font-bold text-zinc-100 capitalize">{channel.platform}</p>
                            <p class="text-[11px] text-zinc-500 font-medium">
                                {channel.paired ? 'Currently linked' : 'Not connected yet'}
                            </p>
                        </div>
                    </div>
                    
                    {#if channel.paired}
                        <div class="flex items-center gap-1.5 px-3 py-1 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                            <Check size={12} strokeWidth={3} />
                            <span class="text-[10px] font-black uppercase tracking-widest">Linked</span>
                        </div>
                    {:else}
                        <button 
                            onclick={() => handlePairing(channel.platform)}
                            class="px-4 py-1.5 rounded-lg bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-300 text-xs font-bold transition-all active:scale-95"
                        >
                            Connect
                        </button>
                    {/if}
                </div>
            {/each}
        </div>
    </div>
{/snippet}

{#snippet whatsappBotSetupSnippet()}
    <div class="space-y-6 py-2">
        <div class="bg-zinc-950 border border-zinc-800 rounded-xl p-8 flex flex-col items-center gap-6">
            <p class="text-xs text-zinc-500 uppercase font-bold tracking-widest">Scan to Connect Bot</p>
            
            <div class="relative group">
                <QRCode data={whatsappQr} size={220} />
                {#if isLoadingQr}
                    <div class="absolute inset-0 bg-zinc-950/80 rounded-lg flex items-center justify-center backdrop-blur-sm">
                        <div class="w-8 h-8 border-4 border-zinc-800 border-t-emerald-500 rounded-full animate-spin"></div>
                    </div>
                {/if}
            </div>

            <button 
                onclick={fetchWhatsappQr}
                disabled={isLoadingQr}
                class="flex items-center gap-2 px-4 py-2 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 rounded-lg text-xs text-zinc-300 transition-all disabled:opacity-50"
            >
                <RefreshCw size={14} class={isLoadingQr ? 'animate-spin' : ''} />
                <span>Refresh QR Code</span>
            </button>
        </div>

        <div class="space-y-3 px-1">
            <p class="text-xs text-zinc-400 font-bold uppercase tracking-wider">Bot Instructions</p>
            <p class="text-sm text-zinc-500 leading-relaxed">
                Scanning this QR code links your WhatsApp account to our backend service. This allows Arta to send and receive messages as you.
            </p>
            <ol class="text-sm text-zinc-400 space-y-2 list-decimal list-inside mt-2">
                <li>Open WhatsApp on your phone</li>
                <li>Go to <span class="text-zinc-200">Linked Devices</span></li>
                <li>Scan this QR code</li>
            </ol>
        </div>
    </div>
{/snippet}

{#snippet pairingContent()}
    <div class="space-y-6 py-2">
        <!-- Nomi Internal Pairing Code -->
        <div class="bg-zinc-950 border border-zinc-800 rounded-xl p-6 flex flex-col items-center gap-4">
            <p class="text-xs text-zinc-500 uppercase font-bold tracking-widest">Internal Pairing Code</p>
            <div class="text-5xl font-black text-emerald-400 tracking-[0.2em] font-mono">
                {pairingCode}
            </div>
            <button 
                onclick={copyToClipboard}
                class="flex items-center gap-2 px-4 py-2 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 rounded-lg text-xs text-zinc-300 transition-all"
            >
                {#if copied}
                    <Check size={14} class="text-emerald-400" />
                    <span class="text-emerald-400">Copied!</span>
                {:else}
                    <Copy size={14} />
                    <span>Copy Code</span>
                {/if}
            </button>
        </div>

        <div class="space-y-3">
            <p class="text-xs text-zinc-400 font-bold uppercase tracking-wider">Instructions</p>
            <ol class="text-sm text-zinc-400 space-y-2 list-decimal list-inside">
                {#if currentPlatform === 'telegram'}
                    <li>Open <a href="https://t.me/ArtaOpenAgentBot" target="_blank" class="text-emerald-400 hover:underline">@ArtaOpenAgentBot</a></li>
                {:else}
                    <li>Open our bot on WhatsApp</li>
                {/if}
                <li>Send the command: <code class="bg-zinc-900 px-1.5 py-0.5 rounded text-emerald-400 font-mono">/pair {pairingCode}</code></li>
                <li>Wait for confirmation here</li>
            </ol>
        </div>
    </div>
{/snippet}

{#snippet pairingFooter()}
    <div class="flex justify-center w-full">
        <button
            onclick={() => popupStore.closeLast()}
            class="px-8 py-2 text-xs font-bold uppercase tracking-wider text-zinc-500 hover:text-zinc-200 transition-all"
        >
            Cancel
        </button>
    </div>
{/snippet}

<header class="h-14 border-b border-zinc-800/50 flex justify-between items-center px-4 bg-[#09090b] sticky top-0 z-20">
    <div class="flex items-center gap-6">
        <!-- Breadcrumbs -->
        <div class="flex items-center gap-2 text-zinc-500">
            <span class="text-xs font-medium hover:text-zinc-300 cursor-pointer transition-colors">Workspace</span>
            <ChevronRight class="w-3.5 h-3.5 text-zinc-700" />
            <span class="text-xs font-semibold text-zinc-200">{conversationStore.activeConversation?.name || 'No Session'}</span>
        </div>

        <!-- Vertical Divider -->
        <div class="h-4 w-[1px] bg-zinc-800"></div>

        <!-- Tabs -->
        <nav class="flex items-center gap-1">
            {#each tabs as tab}
                <a
                    href={tab.href}
                    class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all duration-200 {activeTab === tab.name ? 'bg-zinc-800/50 text-emerald-400 border border-emerald-500/20' : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-900'}"
                >
                    <tab.icon class="w-3.5 h-3.5" />
                    {tab.name}
                </a>
            {/each}
        </nav>
    </div>

    <div class="flex items-center gap-3">
        {#if conversationStore.activeConversationId}
        <!-- App Connection Button -->
        <button 
            class="flex items-center gap-2 px-3 py-1.5 rounded-lg transition-all duration-200 {isPaired ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400 hover:bg-emerald-500/20' : 'bg-zinc-900 border-zinc-800 text-zinc-400 hover:bg-zinc-800'}"
            onclick={openConnectionManager}
            title="App Connections"
        >
            {#if isPaired}
                <Check class="w-3.5 h-3.5" />
                <span class="text-xs font-bold uppercase tracking-wider">Linked App</span>
            {:else}
                <Link class="w-3.5 h-3.5" />
                <span class="text-xs font-bold uppercase tracking-wider">Link App</span>
            {/if}
        </button>

        <!-- Soul Settings Button -->
        <button 
            class="flex items-center justify-center w-8 h-8 rounded-lg hover:bg-zinc-800 transition-colors text-zinc-400 hover:text-zinc-200"
            onclick={openTimeline}
            title="Soul Settings"
        >
            <Settings2 class="w-4 h-4" />
        </button>
        {/if}

        <!-- Status Badge -->
        <div class="flex items-center gap-2 px-3 py-1.5 rounded-full {isGatewayOnline ? 'bg-emerald-500/5 border-emerald-500/10' : 'bg-rose-500/5 border-rose-500/10'} border">
            <div class="relative flex h-2 w-2">
                {#if isGatewayOnline}
                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                    <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                {:else}
                    <span class="relative inline-flex rounded-full h-2 w-2 bg-rose-500"></span>
                {/if}
            </div>
            <span class="text-[10px] {isGatewayOnline ? 'text-emerald-500/90' : 'text-rose-500/90'} font-bold uppercase tracking-widest">
                Gateway {isGatewayOnline ? 'Online' : 'Offline'}
            </span>
        </div>

        <!-- Model Badge -->
        <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-zinc-900 border border-zinc-800/50">
            <Cpu class="w-3.5 h-3.5 text-zinc-500"/>
            <span class="text-[10px] text-zinc-400 font-bold uppercase tracking-widest">{modelInfo.model} {modelInfo.version}</span>
        </div>
    </div>
</header>
