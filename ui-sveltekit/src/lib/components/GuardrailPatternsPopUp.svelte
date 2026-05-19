<script lang="ts">
    import { onMount } from 'svelte';
    import { chatApi } from '$lib/api/client';
    import { Search, ShieldAlert, Plus, Trash2, Loader2, X, AlertTriangle } from 'lucide-svelte';
    import { popupStore } from '$lib/stores/popup.svelte';

    interface PatternInfo {
        id: string;
        content: string;
        created_at: string;
    }

    let patterns = $state<PatternInfo[]>([]);
    let searchQuery = $state('');
    let isLoading = $state(false);
    let isInserting = $state(false);
    let newPattern = $state('');
    let errorMessage = $state<string | null>(null);

    let filteredPatterns = $derived(
        patterns.filter(p => 
            p.content.toLowerCase().includes(searchQuery.toLowerCase())
        )
    );

    async function fetchPatterns() {
        isLoading = true;
        try {
            const res = await chatApi.getGuardrailPatterns();
            patterns = res.data;
        } catch (e) {
            console.error(e);
        } finally {
            isLoading = false;
        }
    }

    async function handleAddPattern() {
        if (!newPattern.trim()) return;
        
        isInserting = true;
        errorMessage = null;
        try {
            const res = await chatApi.insertGuardrailPattern(newPattern.trim());
            if (res.meta.code >= 200 && res.meta.code <= 299) {
                newPattern = '';
                await fetchPatterns();
            } else {
                errorMessage = res.meta.message;
            }
        } catch (e: any) {
            errorMessage = e.message || "Failed to insert pattern";
        } finally {
            isInserting = false;
        }
    }

    async function handleDeletePattern(id: string) {
        try {
            await chatApi.deleteGuardrailPattern(id);
            patterns = patterns.filter(p => p.id !== id);
        } catch (e) {
            console.error(e);
        }
    }

    onMount(() => {
        fetchPatterns();
    });
</script>

<div class="flex flex-col h-full bg-slate-900 text-slate-100 overflow-hidden rounded-lg">
    <!-- Header with Search & Add -->
    <div class="p-4 border-b border-slate-800 space-y-4">
        <div class="flex gap-2">
            <div class="relative flex-1">
                <Plus class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
                <input
                    type="text"
                    bind:value={newPattern}
                    onkeydown={(e) => e.key === 'Enter' && handleAddPattern()}
                    placeholder="Enter new injection pattern to block..."
                    class="w-full bg-slate-950 border border-slate-800 rounded-md py-2 pl-10 pr-4 text-sm focus:outline-none focus:ring-1 focus:ring-rose-500 transition-all"
                />
            </div>
            <button
                onclick={handleAddPattern}
                disabled={isInserting || !newPattern.trim()}
                class="px-4 py-2 bg-rose-600 hover:bg-rose-500 disabled:opacity-50 disabled:cursor-not-allowed rounded-md text-xs font-bold uppercase tracking-wider transition-all flex items-center gap-2"
            >
                {#if isInserting}
                    <Loader2 class="w-3 h-3 animate-spin" />
                {:else}
                    <Plus class="w-3 h-3" />
                {/if}
                Add
            </button>
        </div>

        {#if errorMessage}
            <div class="flex items-center gap-3 p-3 bg-rose-500/10 border border-rose-500/20 rounded-lg animate-in fade-in slide-in-from-top-1">
                <AlertTriangle class="w-4 h-4 text-rose-500 shrink-0" />
                <p class="text-xs text-rose-200 flex-1 font-medium">{errorMessage}</p>
                <button onclick={() => errorMessage = null} class="text-rose-400 hover:text-rose-200">
                    <X class="w-3 h-3" />
                </button>
            </div>
        {/if}

        <div class="relative">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
            <input
                type="text"
                bind:value={searchQuery}
                placeholder="Search blocked patterns..."
                class="w-full bg-slate-950 border border-slate-800 rounded-md py-2 pl-10 pr-4 text-sm focus:outline-none focus:ring-1 focus:ring-sky-500 transition-all"
            />
        </div>
    </div>

    <!-- Patterns List -->
    <div class="flex-1 overflow-y-auto custom-scrollbar p-2">
        {#if isLoading}
            <div class="flex flex-col items-center justify-center h-full space-y-3">
                <Loader2 class="w-8 h-8 text-rose-500 animate-spin" />
                <p class="text-slate-400 text-sm italic font-medium">Scanning security database...</p>
            </div>
        {:else if filteredPatterns.length === 0}
            <div class="flex flex-col items-center justify-center h-full text-slate-500 space-y-2">
                <ShieldAlert class="w-12 h-12 opacity-20" />
                <p class="text-sm">No security patterns found</p>
            </div>
        {:else}
            <div class="grid gap-2">
                {#each filteredPatterns as pattern (pattern.id)}
                    <div class="group p-3 rounded-lg bg-slate-800/30 border border-slate-700/50 hover:bg-slate-800/50 hover:border-rose-500/30 transition-all cursor-default">
                        <div class="flex items-center justify-between gap-4">
                            <div class="flex-1 min-w-0">
                                <p class="text-sm text-slate-200 font-mono break-all leading-relaxed">
                                    {pattern.content}
                                </p>
                                <p class="text-[10px] text-slate-500 mt-1 uppercase tracking-tighter">
                                    Added {new Date(pattern.created_at).toLocaleDateString()}
                                </p>
                            </div>
                            <button
                                onclick={() => handleDeletePattern(pattern.id)}
                                class="p-2 text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 rounded-lg transition-all opacity-0 group-hover:opacity-100"
                                title="Delete Pattern"
                            >
                                <Trash2 class="w-4 h-4" />
                            </button>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>

    <!-- Footer Stats -->
    <div class="p-3 bg-slate-950/50 border-t border-slate-800 flex items-center justify-between">
        <div class="flex items-center gap-2 text-[10px] text-slate-500 font-medium uppercase tracking-widest">
            <ShieldAlert class="w-3 h-3 text-rose-500" />
            <span>{filteredPatterns.length} Blocked Vectors</span>
        </div>
        <div class="text-[10px] text-slate-600 font-mono">
            SECURITY_V1
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
