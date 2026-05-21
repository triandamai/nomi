<script lang="ts">
    import { Hash, X, Plus, Info } from 'lucide-svelte';

    let { 
        intents = $bindable([]), 
        onSave = () => {} 
    } = $props<{
        intents: string[];
        onSave?: () => void;
    }>();

    let inputValue = $state('');

    function addIntents() {
        if (!inputValue.trim()) return;

        // Split by space or comma, trim, and filter empty/duplicates
        const newIntents = inputValue
            .split(/[\s,]+/)
            .map(i => i.trim().toUpperCase())
            .filter(i => i && !intents.includes(i));

        intents = [...intents, ...newIntents];
        inputValue = '';
    }

    function removeIntent(index: number) {
        intents = intents.filter((_val: string, i: number) => i !== index);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter') {
            e.preventDefault();
            addIntents();
        }
    }
</script>

<div class="flex flex-col h-[500px] bg-slate-900 text-slate-100 overflow-hidden rounded-lg">
    <!-- Input Section -->
    <div class="p-6 border-b border-slate-800 bg-slate-950/30">
        <label class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-500 mb-3" for="intent-input">
            <Plus class="w-3 h-3" /> Add New Triggers
        </label>
        <div class="relative">
            <Hash class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
            <input
                id="intent-input"
                type="text"
                bind:value={inputValue}
                onkeydown={handleKeydown}
                placeholder="Type intent name (space or enter to add)..."
                class="w-full bg-slate-900 border border-slate-800 rounded-xl py-3 pl-10 pr-4 text-sm focus:outline-none focus:border-sky-500 transition-all font-mono text-emerald-400 placeholder:text-slate-600"
                autofocus
            />
        </div>
        <div class="mt-3 flex items-start gap-2 text-slate-600">
            <Info class="w-3 h-3 mt-0.5" />
            <p class="text-[10px] italic leading-tight">Semantic triggers tell Nomi when to use this plugin. Use uppercase labels like 'FINANCE' or 'WEATHER'.</p>
        </div>
    </div>

    <!-- Chips Section -->
    <div class="flex-1 p-6 overflow-y-auto custom-scrollbar">
        <div class="flex items-center justify-between mb-4">
            <span class="text-[10px] font-black uppercase tracking-widest text-slate-500">Active Routing Intents</span>
            <span class="text-[10px] font-mono text-slate-600">{intents.length} TOTAL</span>
        </div>

        {#if intents.length === 0}
            <div class="h-full flex flex-col items-center justify-center text-slate-800 space-y-2 opacity-50">
                <Hash class="w-12 h-12" />
                <p class="text-xs font-bold uppercase tracking-widest">No Intents Configured</p>
            </div>
        {:else}
            <div class="flex flex-wrap gap-2">
                {#each intents as intent, i}
                    <div class="flex items-center gap-2 px-3 py-1.5 bg-sky-500/10 border border-sky-500/20 rounded-full group transition-all hover:border-sky-500/50 hover:bg-sky-500/20">
                        <span class="text-[11px] font-black font-mono text-sky-400">#{intent}</span>
                        <button 
                            onclick={() => removeIntent(i)}
                            class="p-0.5 rounded-full hover:bg-rose-500/20 hover:text-rose-400 text-slate-600 transition-all"
                        >
                            <X class="w-3 h-3" />
                        </button>
                    </div>
                {/each}
            </div>
        {/if}
    </div>

    <!-- Action Footer -->
    <div class="p-4 bg-slate-900 border-t border-slate-800 flex justify-end">
        <button 
            onclick={onSave}
            class="px-8 py-2.5 bg-emerald-500 hover:bg-emerald-400 text-slate-950 rounded-lg font-black text-xs uppercase tracking-[0.2em] transition-all shadow-lg shadow-emerald-900/20"
        >
            Done
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
