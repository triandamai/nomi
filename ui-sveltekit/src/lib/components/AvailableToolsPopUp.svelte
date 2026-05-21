<script lang="ts">
    import { onMount } from 'svelte';
    import { chatApi } from '$lib/api/client';
    import { Search, Terminal, Loader2, Sparkles, Beaker } from 'lucide-svelte';
    import { profileStore } from '$lib/stores/profile.svelte';
    import { popupStore } from '$lib/stores/popup.svelte';
    import SkillTesterPopUp from './SkillTesterPopUp.svelte';

    interface ToolInfo {
        name: string;
        description: string;
        intents: string[];
        schema?: any;
    }

    let tools = $state<ToolInfo[]>([]);
    let searchQuery = $state('');
    let isLoading = $state(false);
    let activeTestSchema = $state<any>(null);

    let filteredTools = $derived(
        tools.filter(tool => 
            tool.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            tool.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
            tool.intents.some(intent => intent.toLowerCase().includes(searchQuery.toLowerCase()))
        )
    );

    const isAdmin = $derived(profileStore.currentUser?.role === 'admin');

    async function fetchTools() {
        isLoading = true;
        try {
            // Fetch both available tools AND full schemas
            const [toolsRes, schemasRes] = await Promise.all([
                chatApi.getAvailableTools(),
                chatApi.getSkillSchemas()
            ]);
            
            // Map schemas back to the tools list for testing
            tools = toolsRes.data.map(t => ({
                ...t,
                schema: schemasRes.data.find(s => s.name === t.name)
            }));
        } catch (e) {
            console.error(e);
        } finally {
            isLoading = false;
        }
    }

    function openTester(tool: ToolInfo) {
        if (!tool.schema) return;
        activeTestSchema = tool.schema;
        popupStore.open({
            title: `Testing ${tool.name}`,
            width: 'max-w-2xl',
            contentSnippet: testerSnippet
        });
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
                class="w-full bg-slate-950 border border-slate-800 rounded-md py-2 pl-10 pr-4 text-sm focus:outline-none focus:ring-1 focus:ring-sky-500 transition-all placeholder:text-slate-600"
            />
        </div>
    </div>

    <!-- Tools List -->
    <div class="flex-1 overflow-y-auto custom-scrollbar p-2">
        {#if isLoading}
            <div class="flex flex-col items-center justify-center h-full space-y-3 opacity-50">
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
                    <div class="group p-4 rounded-lg bg-slate-800/40 border border-slate-700/50 hover:bg-slate-800/60 hover:border-sky-500/30 transition-all cursor-default">
                        <div class="flex items-start justify-between gap-4">
                            <div class="flex-1 min-w-0">
                                <div class="flex items-center gap-2 mb-1.5">
                                    <h3 class="font-bold text-sky-400 truncate text-sm uppercase tracking-wider">{tool.name.replace(/_/g, ' ')}</h3>
                                    <span class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-sky-500/10 text-sky-500 border border-sky-500/20 uppercase">
                                        Active
                                    </span>
                                </div>
                                <p class="text-xs text-slate-300 leading-relaxed mb-4 line-clamp-2 italic">
                                    {tool.description}
                                </p>
                                
                                <div class="flex flex-wrap gap-1.5">
                                    {#each tool.intents as intent}
                                        <span class="px-2 py-0.5 rounded-full text-[10px] bg-slate-950 text-slate-500 border border-slate-800 font-mono tracking-tighter">
                                            #{intent}
                                        </span>
                                    {/each}
                                </div>
                            </div>

                            {#if isAdmin}
                                <button
                                    onclick={() => openTester(tool)}
                                    class="p-2.5 bg-slate-900 border border-slate-700 rounded-lg text-slate-400 hover:text-sky-400 hover:border-sky-500/50 hover:bg-sky-500/5 transition-all shadow-sm"
                                    title="Open Skill Tester"
                                >
                                    <Beaker class="w-4 h-4" />
                                </button>
                            {/if}
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
            <span>{filteredTools.length} Skills Mapped</span>
        </div>
        <div class="text-[10px] text-slate-700 font-mono">
            REFLECTOR_V1
        </div>
    </div>
</div>

{#snippet testerSnippet()}
    <SkillTesterPopUp schema={activeTestSchema} />
{/snippet}

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
