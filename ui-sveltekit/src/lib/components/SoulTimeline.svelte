<script lang="ts">
    import { onMount } from 'svelte';
    import { slide, fly } from 'svelte/transition';
    import { History, ChevronDown, ChevronUp, RotateCcw } from 'lucide-svelte';
    import { soulStore } from '$lib/stores/soul.svelte';

    interface Props {
        conversationId: string;
    }

    let { conversationId }: Props = $props();

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
        <div class="flex items-center justify-center p-8 text-slate-500">
            <History class="w-6 h-6 animate-spin mr-2" />
            <span>Loading timeline...</span>
        </div>
    {:else if soulStore.error}
        <div class="p-4 bg-rose-500/10 text-rose-400 rounded-xl text-sm m-4 border border-rose-500/20 font-medium">
            {soulStore.error}
        </div>
    {:else if soulStore.history.length === 0}
        <div class="flex flex-col items-center justify-center p-12 text-slate-500 gap-3">
            <History class="w-10 h-10 opacity-40 text-slate-600" />
            <p class="text-sm font-medium">No soul history found.</p>
        </div>
    {:else}
        <div class="relative pl-6 space-y-6 before:absolute before:inset-0 before:ml-[11px] before:-translate-x-px before:h-full before:w-0.5 before:bg-gradient-to-b before:from-transparent before:via-slate-800 before:to-transparent">
            {#each soulStore.history as entry, index (entry.version)}
                {@const isActive = index === 0}
                <div 
                    class="relative"
                    in:fly={{ y: 20, duration: 400, delay: index * 100 }}
                >
                    <!-- Timeline Dot -->
                    <div class="absolute -left-6 w-5 h-5 rounded-full border-4 border-slate-950 {isActive ? 'bg-blue-500 shadow-blue-500/50' : 'bg-slate-700'} flex items-center justify-center z-10 shadow-sm transition-colors">
                    </div>

                    <div class="bg-slate-900/50 rounded-2xl p-5 border border-slate-800/50 backdrop-blur-sm transition-all hover:border-slate-700/80">
                        <div class="flex flex-wrap items-start justify-between gap-4 mb-3">
                            <div class="flex items-center gap-3">
                                <span class="text-blue-400 font-mono text-sm font-bold">
                                    v{entry.version}
                                </span>
                                {#if isActive}
                                    <span class="px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-400 text-[9px] font-black uppercase tracking-widest border border-blue-500/20">
                                        Active
                                    </span>
                                {/if}
                            </div>
                            <div class="text-[10px] text-slate-500 font-mono tracking-wider uppercase">
                                {formatDate(entry.created_at)}
                            </div>
                        </div>

                        <p class="text-slate-200 text-sm font-bold mb-4 leading-relaxed">
                            {entry.change_reason}
                        </p>

                        <div class="flex items-center gap-4">
                            <button 
                                class="flex items-center gap-1.5 text-[11px] font-bold uppercase tracking-widest text-slate-400 hover:text-slate-200 transition-colors"
                                onclick={() => toggleDetails(entry.version)}
                            >
                                {#if expandedVersion === entry.version}
                                    <ChevronUp class="w-4 h-4" />
                                    Hide Details
                                {:else}
                                    <ChevronDown class="w-4 h-4" />
                                    Show Details
                                {/if}
                            </button>

                            {#if !isActive}
                                <div class="w-px h-3 bg-slate-800"></div>
                                <button 
                                    class="flex items-center gap-1.5 text-[11px] font-bold uppercase tracking-widest text-blue-400 hover:text-blue-300 transition-colors"
                                    onclick={() => restoreVersion(entry.version)}
                                >
                                    <RotateCcw class="w-4 h-4" />
                                    Restore
                                </button>
                            {/if}
                        </div>

                        {#if expandedVersion === entry.version}
                            <div 
                                class="mt-4 p-4 bg-slate-950/80 rounded-xl border border-slate-800/80 text-xs text-slate-300 font-mono overflow-x-auto whitespace-pre-wrap leading-relaxed"
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
