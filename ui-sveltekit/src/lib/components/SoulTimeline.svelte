<script lang="ts">
    import { onMount } from 'svelte';
    import { slide, fly } from 'svelte/transition';
    import { History, ChevronDown, ChevronUp, RotateCcw } from 'lucide-svelte';
    import { soulStore } from '$lib/stores/soul.svelte';

    interface Props {
        conversationId: string;
    }

    let { conversationId } = $props<Props>();

    let expandedVersion = $state<number | null>(null);

    onMount(async () => {
        await soulStore.loadHistory(conversationId);
    });

    async function restoreVersion(version: number) {
        try {
            await soulStore.restoreVersion(conversationId, version);
        } catch (e: any) {
            alert(e.message || 'Failed to restore soul');
        }
    }

    function toggleDetails(version: number) {
        expandedVersion = expandedVersion === version ? null : version;
    }

    function formatDate(dateStr: string) {
        try {
            const date = new Date(dateStr);
            return new Intl.DateTimeFormat('en-US', {
                month: 'short', day: 'numeric', hour: 'numeric', minute: 'numeric'
            }).format(date);
        } catch (e) {
            return dateStr;
        }
    }
</script>

<div class="relative w-full h-full flex flex-col">
    {#if soulStore.loading}
        <div class="flex items-center justify-center p-8 text-zinc-500">
            <History class="w-6 h-6 animate-spin mr-2" />
            <span>Loading timeline...</span>
        </div>
    {:else if soulStore.error}
        <div class="p-4 bg-red-500/10 text-red-400 rounded-lg text-sm m-4 border border-red-500/20">
            {soulStore.error}
        </div>
    {:else if soulStore.history.length === 0}
        <div class="flex flex-col items-center justify-center p-8 text-zinc-500 gap-2">
            <History class="w-8 h-8 opacity-50" />
            <p class="text-sm">No soul history found.</p>
        </div>
    {:else}
        <div class="relative pl-6 space-y-6 before:absolute before:inset-0 before:ml-[11px] before:-translate-x-px md:before:mx-auto md:before:translate-x-0 before:h-full before:w-0.5 before:bg-gradient-to-b before:from-transparent before:via-slate-800 before:to-transparent">
            {#each soulStore.history as entry, index (entry.version)}
                {@const isActive = index === 0}
                <div 
                    class="relative"
                    in:fly={{ y: 20, duration: 400, delay: index * 100 }}
                >
                    <!-- Timeline Dot -->
                    <div class="absolute -left-6 w-5 h-5 rounded-full border-4 border-zinc-950 {isActive ? 'bg-emerald-500' : 'bg-slate-700'} flex items-center justify-center z-10 shadow-sm">
                    </div>

                    <div class="bg-slate-900/50 rounded-xl p-4 border border-zinc-800/50 backdrop-blur-sm transition-all hover:border-zinc-700/50">
                        <div class="flex flex-wrap items-start justify-between gap-4 mb-2">
                            <div class="flex items-center gap-3">
                                <span class="text-blue-400 font-mono text-sm font-semibold">
                                    v{entry.version}
                                </span>
                                {#if isActive}
                                    <span class="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 text-[10px] font-bold uppercase tracking-wider border border-emerald-500/20">
                                        Active
                                    </span>
                                {/if}
                            </div>
                            <div class="text-xs text-zinc-500 font-medium">
                                {formatDate(entry.created_at)}
                            </div>
                        </div>

                        <p class="text-zinc-200 text-sm font-bold mb-3">
                            {entry.change_reason}
                        </p>

                        <div class="flex items-center gap-3">
                            <button 
                                class="flex items-center gap-1.5 text-xs font-medium text-zinc-400 hover:text-zinc-200 transition-colors"
                                onclick={() => toggleDetails(entry.version)}
                            >
                                {#if expandedVersion === entry.version}
                                    <ChevronUp class="w-3.5 h-3.5" />
                                    Hide Details
                                {:else}
                                    <ChevronDown class="w-3.5 h-3.5" />
                                    Show Details
                                {/if}
                            </button>

                            {#if !isActive}
                                <div class="w-px h-3 bg-zinc-800"></div>
                                <button 
                                    class="flex items-center gap-1.5 text-xs font-medium text-blue-400 hover:text-blue-300 transition-colors"
                                    onclick={() => restoreVersion(entry.version)}
                                >
                                    <RotateCcw class="w-3.5 h-3.5" />
                                    Restore
                                </button>
                            {/if}
                        </div>

                        {#if expandedVersion === entry.version}
                            <div 
                                class="mt-4 p-3 bg-zinc-950/50 rounded-lg border border-zinc-800/50 text-sm text-zinc-300 font-mono overflow-x-auto whitespace-pre-wrap"
                                transition:slide={{ duration: 200 }}
                            >
                                {entry.soul_content}
                            </div>
                        {/if}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
