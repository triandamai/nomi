<script lang="ts">
    import { ChevronRight, MessageSquare, Network, Menu, X, Cpu, Info, Palette, Check } from 'lucide-svelte';
    import { slide } from 'svelte/transition';
    import { page } from '$app/state';
    import { conversationStore } from '$lib/stores/conversation.svelte';
    import { headerStore } from '$lib/stores/header.svelte';
    import { themeStore } from '$lib/stores/theme.svelte';
    import { formatTokenCount } from '$lib/utils';
    import { onMount } from "svelte";
    import { popupStore } from '$lib/stores/popup.svelte';
    import ThreadDetailPopUp from './ThreadDetailPopUp.svelte';

    let isThemeMenuOpen = $state(false);

    let { title = '' } = $props<{ title?: string }>();

    let activeTab = $derived(page.url.pathname === '/rag' ? 'RAG' : 'Chat');
    let isMenuOpen = $state(false);

    onMount(() => {
        headerStore.init();
    });

    function openThreadDetail() {
        popupStore.open({
            title: 'Thread Detail',
            width: 'max-w-md',
            contentSnippet: threadDetailSnippet
        });
    }

    const tabs = [
        { name: 'Chat', href: '/chat', icon: MessageSquare },
        { name: 'RAG', href: '/rag', icon: Network }
    ];
</script>

{#snippet threadDetailSnippet()}
    <ThreadDetailPopUp />
{/snippet}

<header class="h-14 border-b border-slate-800/50 flex justify-between items-center px-4 bg-[#0f172a] sticky top-0 z-20">
    <div class="flex items-center gap-6">
        <!-- Breadcrumbs -->
        <div class="flex items-center gap-2 text-slate-500">
            <span class="text-xs font-medium hover:text-slate-300 cursor-pointer transition-colors hidden sm:inline">Workspace</span>
            <ChevronRight class="w-3.5 h-3.5 text-slate-700 hidden sm:block" />
            <button 
                onclick={openThreadDetail}
                class="flex items-center gap-2 text-xs font-semibold text-slate-200 hover:text-blue-400 transition-colors truncate max-w-[150px] sm:max-w-none group text-left"
            >
                {conversationStore.activeConversation?.name || 'No Session'}
                <Info size={12} class="text-slate-600 group-hover:text-blue-500 transition-colors shrink-0" />
            </button>
            <div class="h-3 w-[1px] bg-slate-800 mx-1"></div>
            <span class="text-[10px] font-mono text-slate-500">
                {formatTokenCount(conversationStore.activeConversation?.cumulative_tokens)}
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

        <!-- Theme Picker -->
        <div class="relative">
            {#if isThemeMenuOpen}
                <div class="fixed inset-0 z-40" onclick={() => isThemeMenuOpen = false}></div>
            {/if}

            <button 
                onclick={() => isThemeMenuOpen = !isThemeMenuOpen}
                class="flex items-center justify-center p-2 rounded-full bg-slate-900 border border-slate-800/50 text-slate-400 hover:text-blue-400 hover:scale-105 active:scale-95 transition-all duration-200 cursor-pointer min-h-[32px] min-w-[32px] relative z-50"
                aria-label="Switch Theme"
                title="Change appearance theme"
            >
                <Palette class="w-3.5 h-3.5" />
            </button>

            {#if isThemeMenuOpen}
                <div 
                    transition:slide={{ duration: 150 }}
                    class="absolute right-0 top-full mt-2 w-72 p-3 bg-slate-950/95 backdrop-blur-xl border border-slate-800/80 rounded-2xl shadow-2xl z-50 flex flex-col gap-2"
                >
                    <p class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500 mb-1 border-b border-slate-800/50 pb-1.5 px-1">Appearance Themes</p>
                    <div class="flex flex-col gap-1 max-h-[320px] overflow-y-auto custom-scrollbar">
                        {#each themeStore.palettes as palette}
                            <button
                                onclick={() => {
                                    themeStore.setTheme(palette.id);
                                    isThemeMenuOpen = false;
                                }}
                                class="flex items-center justify-between p-2 rounded-xl text-left hover:bg-slate-900/60 transition-all duration-200 border {themeStore.currentTheme === palette.id ? 'border-blue-500/30 bg-blue-500/5' : 'border-transparent'}"
                            >
                                <div class="flex items-center gap-3">
                                    <!-- Palette circle color preview -->
                                    <div class="flex h-5 w-5 rounded-full overflow-hidden border border-slate-800 shrink-0">
                                        <div class="w-1/2 h-full" style="background-color: {palette.primaryColor}"></div>
                                        <div class="w-1/2 h-full" style="background-color: {palette.accentColor}"></div>
                                    </div>
                                    <div class="flex flex-col">
                                        <span class="text-xs font-semibold {themeStore.currentTheme === palette.id ? 'text-blue-400' : 'text-slate-200'}">{palette.name}</span>
                                        <span class="text-[9px] text-slate-500 truncate max-w-[170px]">{palette.description}</span>
                                    </div>
                                </div>
                                {#if themeStore.currentTheme === palette.id}
                                    <Check class="w-3.5 h-3.5 text-blue-400 shrink-0" />
                                {/if}
                            </button>
                        {/each}
                    </div>
                </div>
            {/if}
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

        <!-- Mobile Theme Switcher -->
        <div class="space-y-3">
            <p class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500 px-2">Appearance Theme</p>
            <div class="grid grid-cols-2 gap-2">
                {#each themeStore.palettes as palette}
                    <button
                        onclick={() => {
                            themeStore.setTheme(palette.id);
                        }}
                        class="flex flex-col items-center justify-center p-3 rounded-2xl border text-center transition-all duration-200 {themeStore.currentTheme === palette.id ? 'border-blue-500 bg-blue-500/5 text-blue-400 font-bold' : 'border-slate-800 bg-slate-900/30 text-slate-400 hover:bg-slate-900/50'}"
                    >
                        <div class="flex h-5 w-5 rounded-full overflow-hidden border border-slate-800 mb-2">
                            <div class="w-1/2 h-full" style="background-color: {palette.primaryColor}"></div>
                            <div class="w-1/2 h-full" style="background-color: {palette.accentColor}"></div>
                        </div>
                        <span class="text-xs">{palette.name}</span>
                    </button>
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
