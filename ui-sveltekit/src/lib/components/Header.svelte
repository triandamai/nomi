<script lang="ts">
    import { ChevronRight, MessageSquare, Network, Menu, X, Cpu } from 'lucide-svelte';
    import { slide } from 'svelte/transition';
    import { page } from '$app/state';
    import { conversationStore } from '$lib/stores/conversation.svelte';
    import { headerStore } from '$lib/stores/header.svelte';
    import { formatTokenCount } from '$lib/utils';
    import { onMount } from "svelte";

    let { title = '' } = $props<{ title?: string }>();

    let activeTab = $derived(page.url.pathname === '/rag' ? 'RAG' : 'Chat');
    let isMenuOpen = $state(false);

    onMount(() => {
        headerStore.init();
    });

    const tabs = [
        { name: 'Chat', href: '/chat', icon: MessageSquare },
        { name: 'RAG', href: '/rag', icon: Network }
    ];
</script>

<header class="h-14 border-b border-slate-800/50 flex justify-between items-center px-4 bg-[#0f172a] sticky top-0 z-20">
    <div class="flex items-center gap-6">
        <!-- Breadcrumbs -->
        <div class="flex items-center gap-2 text-slate-500">
            <span class="text-xs font-medium hover:text-slate-300 cursor-pointer transition-colors hidden sm:inline">Workspace</span>
            <ChevronRight class="w-3.5 h-3.5 text-slate-700 hidden sm:block" />
            <span class="text-xs font-semibold text-slate-200 truncate max-w-[120px] sm:max-w-none">
                {conversationStore.activeConversation?.name || 'No Session'} - <span class="font-mono">{formatTokenCount(conversationStore.activeConversation?.cumulative_tokens)}</span> Token
            </span>
        </div>

        <!-- Vertical Divider -->
        <div class="h-4 w-[1px] bg-slate-800 hidden md:block"></div>

        <!-- Tabs -->
        <nav class="hidden md:flex items-center gap-1">
            {#each tabs as tab}
                <a
                    href={tab.href}
                    class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all duration-200 {activeTab === tab.name ? 'bg-blue-600/10 text-blue-400 border border-blue-500/20' : 'text-slate-500 hover:text-slate-300 hover:bg-slate-900'}"
                >
                    <tab.icon class="w-3.5 h-3.5" />
                    {tab.name}
                </a>
            {/each}
        </nav>
    </div>

    <div class="hidden md:flex items-center gap-3">
        <!-- Status Badge -->
        <div class="flex items-center gap-2 px-3 py-1.5 rounded-full {headerStore.isGatewayOnline ? 'bg-emerald-500/5 border-emerald-500/10' : 'bg-rose-500/5 border-rose-500/10'} border">
            <div class="relative flex h-2 w-2">
                {#if headerStore.isGatewayOnline}
                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                    <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                {:else}
                    <span class="relative inline-flex rounded-full h-2 w-2 bg-rose-500"></span>
                {/if}
            </div>
            <span class="text-[10px] {headerStore.isGatewayOnline ? 'text-emerald-500/90' : 'text-rose-500/90'} font-bold uppercase tracking-widest">
                Gateway {headerStore.isGatewayOnline ? 'Online' : 'Offline'}
            </span>
        </div>

        <!-- Model Badge -->
        <div class="relative group">
            <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-slate-900 border border-slate-800/50 cursor-help">
                <Cpu class="w-3.5 h-3.5 text-slate-500"/>
                <span class="text-[10px] text-slate-400 font-bold uppercase tracking-widest">{headerStore.modelInfo.agent_model}</span>
            </div>
            
            <!-- Tooltip -->
            <div class="absolute right-0 top-full mt-2 w-64 p-3 bg-slate-950 border border-slate-800 rounded-xl shadow-xl opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-50">
                <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500 mb-2 border-b border-slate-800 pb-1">System Models</p>
                <div class="space-y-2">
                    <div class="flex justify-between items-center">
                        <span class="text-[11px] text-slate-400">Agent:</span>
                        <span class="text-[11px] font-mono text-blue-400">{headerStore.modelInfo.agent_model}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-[11px] text-slate-400">RAG:</span>
                        <span class="text-[11px] font-mono text-emerald-400">{headerStore.modelInfo.rag_embedding}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-[11px] text-slate-400">Classification:</span>
                        <span class="text-[11px] font-mono text-amber-400">{headerStore.modelInfo.media_classification}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-[11px] text-slate-400">Vision:</span>
                        <span class="text-[11px] font-mono text-purple-400">{headerStore.modelInfo.media_analyze}</span>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Mobile Menu Button -->
    <button 
        class="md:hidden p-2 text-zinc-400 hover:text-zinc-200 transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
        onclick={() => isMenuOpen = !isMenuOpen}
        aria-label="Toggle Menu"
    >
        {#if isMenuOpen}
            <X class="w-6 h-6" />
        {:else}
            <Menu class="w-6 h-6" />
        {/if}
    </button>
</header>

{#if isMenuOpen}
    <!-- Mobile Menu Drawer -->
    <div 
        transition:slide
        class="md:hidden fixed inset-x-0 top-14 bottom-0 bg-[#0f172a] z-30 border-t border-slate-800/50 flex flex-col p-6 space-y-8 overflow-y-auto"
    >
        <!-- Mobile Tabs -->
        <div class="space-y-4">
            <p class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500 px-2">Navigation</p>
            <div class="grid gap-2">
                {#each tabs as tab}
                    <a
                        href={tab.href}
                        onclick={() => isMenuOpen = false}
                        class="flex items-center gap-4 px-4 py-4 rounded-2xl text-sm font-bold transition-all {activeTab === tab.name ? 'bg-blue-600/10 text-blue-400 border border-blue-500/20' : 'text-slate-400 hover:bg-slate-900 border border-transparent'}"
                    >
                        <tab.icon class="w-5 h-5" />
                        {tab.name}
                    </a>
                {/each}
            </div>
        </div>

        <!-- Status & Model info at bottom -->
        <div class="mt-auto pt-8 border-t border-slate-800/50 space-y-4">
            <div class="flex flex-col p-5 rounded-3xl bg-slate-900 border border-slate-800 gap-6">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <div class="relative flex h-2.5 w-2.5">
                            {#if headerStore.isGatewayOnline}
                                <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
                            {:else}
                                <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-rose-500"></span>
                            {/if}
                        </div>
                        <span class="text-sm font-black uppercase tracking-tight text-slate-200">Gateway {headerStore.isGatewayOnline ? 'Online' : 'Offline'}</span>
                    </div>
                    <div class="flex items-center gap-2 text-slate-500">
                        <Cpu class="w-4 h-4"/>
                        <span class="text-[10px] font-black uppercase tracking-widest">System</span>
                    </div>
                </div>
                
                <div class="space-y-3 border-t border-slate-800/50 pt-4">
                    <div class="flex justify-between items-center">
                        <span class="text-[11px] font-bold text-slate-500 uppercase tracking-widest">Agent</span>
                        <span class="text-[11px] font-mono text-blue-400 font-bold">{headerStore.modelInfo.agent_model}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-[11px] font-bold text-slate-500 uppercase tracking-widest">RAG</span>
                        <span class="text-[11px] font-mono text-emerald-400 font-bold">{headerStore.modelInfo.rag_embedding}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-[11px] font-bold text-slate-500 uppercase tracking-widest">Classification</span>
                        <span class="text-[11px] font-mono text-amber-400 font-bold">{headerStore.modelInfo.media_classification}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-[11px] font-bold text-slate-500 uppercase tracking-widest">Vision</span>
                        <span class="text-[11px] font-mono text-purple-400 font-bold">{headerStore.modelInfo.media_analyze}</span>
                    </div>
                </div>
            </div>
        </div>
    </div>
{/if}
