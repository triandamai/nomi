<script lang="ts">
    import { Cpu, ChevronRight, MessageSquare, Network, Settings2 } from 'lucide-svelte';
    import { page } from '$app/state';
    import SoulTimeline from './SoulTimeline.svelte';
    import { conversationStore } from '$lib/stores/conversation.svelte';
    import { popupStore } from '$lib/stores/popup.svelte';

    let activeTab = $derived(page.url.pathname === '/rag' ? 'RAG' : 'Chat');

    const tabs = [
        { name: 'Chat', href: '/', icon: MessageSquare },
        { name: 'RAG', href: '/rag', icon: Network }
    ];

    function openTimeline() {
        popupStore.open({
            title: 'Soul Timeline',
            width: 'w-1/3',
            contentSnippet: soulTimelineSnippet
        });
    }
</script>

{#snippet soulTimelineSnippet()}
    {#if conversationStore.activeConversationId}
        <SoulTimeline conversationId={conversationStore.activeConversationId} />
    {/if}
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
