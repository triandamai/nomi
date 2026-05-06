<script lang="ts">
    import {Check, ChevronRight, Copy, Cpu, Link, MessageSquare, Network, Settings2} from 'lucide-svelte';
    import {page} from '$app/state';
    import SoulTimeline from './SoulTimeline.svelte';
    import {conversationStore} from '$lib/stores/conversation.svelte';
    import {popupStore} from '$lib/stores/popup.svelte';
    import {eventBus} from '$lib/utils';

    let activeTab = $derived(page.url.pathname === '/rag' ? 'RAG' : 'Chat');
    let pairingCode = $state('');
    let copied = $state(false);

    const tabs = [
        { name: 'Chat', href: '/', icon: MessageSquare },
        { name: 'RAG', href: '/rag', icon: Network }
    ];

    eventBus.subscribe('sse-pairing-success', (data: any) => {
        if (data.conversation_id === conversationStore.activeConversationId) {
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

    async function handlePairing() {
        if (!conversationStore.activeConversationId) return;
        try {
            const data = await conversationStore.getPairingCode(conversationStore.activeConversationId);
            pairingCode = data.pairing_code;
            popupStore.open({
                title: 'Link Telegram',
                width: 'max-w-md',
                contentSnippet: pairingContent,
                footerSnippet: pairingFooter
            });
        } catch (e) {
            console.error(e);
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

{#snippet pairingContent()}
    <div class="space-y-6 py-2">
        <div class="bg-zinc-950 border border-zinc-800 rounded-xl p-6 flex flex-col items-center gap-4">
            <p class="text-xs text-zinc-500 uppercase font-bold tracking-widest">Your Pairing Code</p>
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
                <li>Open <a href="https://t.me/ArtaOpenAgentBot" target="_blank" class="text-emerald-400 hover:underline">@ArtaOpenAgentBot</a> on Telegram</li>
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
        <!-- Link Telegram Button -->
        <button 
            class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-emerald-500/10 hover:bg-emerald-500/20 border border-emerald-500/20 transition-colors text-emerald-400 hover:text-emerald-300"
            onclick={handlePairing}
            title="Link Telegram"
        >
            <Link class="w-3.5 h-3.5" />
            <span class="text-xs font-bold uppercase tracking-wider">Link App</span>
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
        <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-emerald-500/5 border border-emerald-500/10">
            <div class="relative flex h-2 w-2">
                <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
            </div>
            <span class="text-[10px] text-emerald-500/90 font-bold uppercase tracking-widest">Gateway Live</span>
        </div>

        <!-- Model Badge -->
        <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-zinc-900 border border-zinc-800/50">
            <Cpu class="w-3.5 h-3.5 text-zinc-500"/>
            <span class="text-[10px] text-zinc-400 font-bold uppercase tracking-widest">Llama 3.1 70B</span>
        </div>
    </div>
</header>
