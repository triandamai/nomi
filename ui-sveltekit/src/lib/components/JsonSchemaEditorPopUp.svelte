<script lang="ts">
    import { onMount } from 'svelte';
    import MonacoEditor from './MonacoEditor.svelte';
    import { FileJson, Eye, AlertCircle, CheckCircle2, Box } from 'lucide-svelte';

    let { 
        schemaJson = $bindable(''), 
        onSave = () => {} 
    } = $props<{
        schemaJson: string;
        onSave?: (json: string) => void;
    }>();

    let localJson = $state(schemaJson);
    let parseError = $state<string | null>(null);
    let parsedProperties = $state<Record<string, any>>({});

    $effect(() => {
        try {
            const parsed = JSON.parse(localJson);
            parsedProperties = parsed.properties || {};
            parseError = null;
        } catch (e: any) {
            parseError = e.message;
        }
    });

    function handleSave() {
        if (parseError) return;
        schemaJson = localJson;
        onSave(localJson);
    }
</script>

<div class="flex flex-col h-[700px] bg-slate-900 text-slate-100 overflow-hidden rounded-lg">
    <!-- Editor Section (Top) -->
    <div class="flex-1 flex flex-col min-h-0">
        <div class="h-10 shrink-0 border-b border-slate-800 flex items-center justify-between px-4 bg-slate-950/50">
            <div class="flex items-center gap-2">
                <FileJson class="w-3.5 h-3.5 text-sky-400" />
                <span class="text-[10px] font-black uppercase tracking-widest text-slate-400">JSON Schema Definition</span>
            </div>
            {#if parseError}
                <div class="flex items-center gap-1.5 text-rose-400 animate-pulse">
                    <AlertCircle class="w-3 h-3" />
                    <span class="text-[9px] font-bold uppercase tracking-tighter">Invalid JSON</span>
                </div>
            {:else}
                <div class="flex items-center gap-1.5 text-emerald-500">
                    <CheckCircle2 class="w-3 h-3" />
                    <span class="text-[9px] font-bold uppercase tracking-tighter">Schema Valid</span>
                </div>
            {/if}
        </div>
        <div class="flex-1 relative bg-[#0d1117]">
            <MonacoEditor bind:value={localJson} language="json" />
        </div>
    </div>

    <!-- Preview Section (Bottom) -->
    <div class="h-64 border-t border-slate-800 bg-slate-950 flex flex-col shrink-0">
        <div class="h-10 shrink-0 border-b border-slate-800/50 flex items-center px-4 bg-slate-900/30">
            <div class="flex items-center gap-2">
                <Eye class="w-3.5 h-3.5 text-slate-500" />
                <span class="text-[10px] font-black uppercase tracking-widest text-slate-500">Property Preview</span>
            </div>
        </div>
        
        <div class="flex-1 p-4 overflow-y-auto custom-scrollbar">
            {#if Object.keys(parsedProperties).length === 0}
                <div class="h-full flex flex-col items-center justify-center text-slate-700 space-y-2 opacity-50">
                    <Box class="w-8 h-8" />
                    <p class="text-[10px] font-bold uppercase tracking-widest">No Properties Defined</p>
                </div>
            {:else}
                <div class="grid grid-cols-2 gap-3">
                    {#each Object.entries(parsedProperties) as [key, prop]}
                        <div class="p-3 bg-slate-900 border border-slate-800 rounded-xl space-y-1.5 group hover:border-sky-500/30 transition-colors">
                            <div class="flex items-center justify-between">
                                <span class="text-xs font-black text-slate-200 font-mono tracking-tight">{key}</span>
                                <span class="px-1.5 py-0.5 rounded bg-slate-800 text-[9px] font-bold text-sky-400 uppercase tracking-tighter border border-slate-700">
                                    {prop.type || 'any'}
                                </span>
                            </div>
                            {#if prop.description}
                                <p class="text-[10px] text-slate-500 line-clamp-2 italic leading-relaxed">{prop.description}</p>
                            {/if}
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    </div>

    <!-- Action Footer -->
    <div class="p-4 bg-slate-900 border-t border-slate-800 flex justify-end">
        <button 
            onclick={handleSave}
            disabled={!!parseError}
            class="px-6 py-2.5 bg-sky-500 hover:bg-sky-400 disabled:opacity-50 disabled:cursor-not-allowed text-slate-950 rounded-lg font-black text-xs uppercase tracking-[0.2em] transition-all shadow-lg shadow-sky-900/20"
        >
            Apply Schema Changes
        </button>
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
</style>
