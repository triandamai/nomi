<script lang="ts">
    import { onMount } from 'svelte';
    import { chatApi } from '$lib/api/client';
    import { Search, Terminal, Info, ChevronRight, Loader2, Sparkles } from 'lucide-svelte';

    interface ToolInfo {
        name: string;
        description: string;
        intents: string[];
    }

    let tools = $state<ToolInfo[]>([]);
    let searchQuery = $state('');
    let isLoading = $state(false);

    let filteredTools = $derived(
        tools.filter(tool => 
            tool.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            tool.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
            tool.intents.some(intent => intent.toLowerCase().includes(searchQuery.toLowerCase()))
        )
    );

    async function fetchTools() {
        isLoading = true;
        try {
            const res = await chatApi.getAvailableTools();
            tools = res.data;
        } catch (e) {
            console.error(e);
        } finally {
            isLoading = false;
        }
    }

    onMount(() => {
        fetchTools();
    });
</script>

<div class="flex flex-col h-full bg-slate-900 text-slate-100 overflow-hidden rounded-lg">
    <!-- Header with Search -->
    <div class="p-4 border-b border-slate-800 space-y-4">
        <div class="relative">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
            <input
                type="text"
                bind:value={searchQuery}
                placeholder="Search tools, descriptions, or intents..."
                class="w-full bg-slate-950 border border-slate-800 rounded-md py-2 pl-10 pr-4 text-sm focus:outline-none focus:ring-1 focus:ring-sky-500 transition-all"
            />
        </div>
    </div>

    <!-- Tools List -->
    <div class="flex-1 overflow-y-auto custom-scrollbar p-2">
        {#if isLoading}
            <div class="flex flex-col items-center justify-center h-full space-y-3">
                <Loader2 class="w-8 h-8 text-sky-500 animate-spin" />
                <p class="text-slate-400 text-sm italic font-medium">Scanning capabilities...</p>
            </div>
        {:else if filteredTools.length === 0}
            <div class="flex flex-col items-center justify-center h-full text-slate-500 space-y-2">
                <Terminal class="w-12 h-12 opacity-20" />
                <p class="text-sm">No tools found matching your query</p>
            </div>
        {:else}
            <div class="grid gap-2">
                {#each filteredTools as tool}
                    <div class="group p-3 rounded-lg bg-slate-800/50 border border-slate-700/50 hover:bg-slate-800 hover:border-sky-500/50 transition-all cursor-default">
                        <div class="flex items-start justify-between gap-3">
                            <div class="flex-1 min-w-0">
                                <div class="flex items-center gap-2 mb-1">
                                    <h3 class="font-bold text-sky-400 truncate text-sm uppercase tracking-wider">{tool.name.replace(/_/g, ' ')}</h3>
                                    <span class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-sky-500/10 text-sky-500 border border-sky-500/20 uppercase">
                                        Active
                                    </span>
                                </div>
                                <p class="text-xs text-slate-300 leading-relaxed mb-3 line-clamp-2 italic">
                                    {tool.description}
                                </p>
                                
                                <div class="flex flex-wrap gap-1.5">
                                    {#each tool.intents as intent}
                                        <span class="px-2 py-0.5 rounded-full text-[10px] bg-slate-950 text-slate-400 border border-slate-800 font-mono">
                                            #{intent}
                                        </span>
                                    {/each}
                                </div>
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>

    <!-- Footer Stats -->
    <div class="p-3 bg-slate-950/50 border-t border-slate-800 flex items-center justify-between">
        <div class="flex items-center gap-2 text-[10px] text-slate-500 font-medium uppercase tracking-widest">
            <Sparkles class="w-3 h-3 text-sky-500" />
            <span>{filteredTools.length} Skills Available</span>
        </div>
        <div class="text-[10px] text-slate-600 font-mono">
            v2.0.0-FLASH
        </div>
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: #1e293b;
        border-radius: 10px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: #334155;
    }
</style>
