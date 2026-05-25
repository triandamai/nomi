<script lang="ts">
    import { onMount } from 'svelte';
    import { adminStore } from '$lib/stores/admin.svelte';
    import { popupStore } from '$lib/stores/popup.svelte';
    import { ChevronDown, Loader2, MessageSquare, Database, Calendar, Edit3, Save, X, Terminal } from 'lucide-svelte';

    // Detail / Edit State
    let selectedConv = $state<any>(null);
    let editMaxTokens = $state<number>(0);
    let editThresholds = $state<any>({
        interaction_gate: 0.6,
        intent_classification: 0.4,
        guardrails: 0.65
    });
    let isSaving = $state(false);

    onMount(() => {
        adminStore.fetchConversations();
    });

    function getInteractionMode(val: number) {
        if (val <= 0.25) return { label: 'Proactive', color: 'text-emerald-400', icon: '🏁' };
        if (val <= 0.50) return { label: 'Balanced', color: 'text-blue-400', icon: '🤝' };
        if (val <= 0.75) return { label: 'Conservative', color: 'text-amber-400', icon: '🛡️' };
        return { label: 'Silent Monitor', color: 'text-slate-400', icon: '🤫' };
    }

    function getIntentMode(val: number) {
        if (val <= 0.40) return { label: 'Experimental', color: 'text-purple-400', icon: '🧪' };
        if (val <= 0.70) return { label: 'Adaptive', color: 'text-blue-400', icon: '🏎️' };
        return { label: 'Strict', color: 'text-rose-400', icon: '📐' };
    }

    function getGuardrailMode(val: number) {
        if (val <= 0.50) return { label: 'Permissive', color: 'text-emerald-400', icon: '🔓' };
        if (val <= 0.80) return { label: 'Standard', color: 'text-blue-400', icon: '👤' };
        return { label: 'Hardened Shield', color: 'text-rose-400', icon: '🌋' };
    }

    function formatTokens(n: number | undefined | null) {
        if (n === undefined || n === null) return '0';
        return new Intl.NumberFormat().format(n);
    }

    function formatDate(dateStr: string) {
        return new Date(dateStr).toLocaleString('id-ID', {
            day: '2-digit',
            month: 'short',
            year: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        }) + ' WIB';
    }

    function getUsagePercentage(current: number, max: number) {
        if (!max || max === 0) return 0;
        return Math.min(Math.round((current / max) * 100), 100);
    }

    function openDetail(conv: any) {
        selectedConv = conv;
        editMaxTokens = conv.max_token_usage || 0;
        editThresholds = conv.gateway_thresholds || {
            interaction_gate: 0.6,
            intent_classification: 0.4,
            guardrails: 0.65
        };
        popupStore.open({
            title: 'Thread Configuration',
            width: 'max-w-md',
            contentSnippet: detailSnippet,
            footerSnippet: detailFooter
        });
    }

    async function handleUpdate() {
        if (!selectedConv) return;
        isSaving = true;
        try {
            await adminStore.updateConversation(selectedConv.id, { 
                max_token_usage: editMaxTokens,
                thresholds: editThresholds
            });
            popupStore.closeLast();
        } catch (e) {
            console.error(e);
        } finally {
            isSaving = false;
        }
    }
</script>

{#snippet detailSnippet()}
    {#if selectedConv}
        <div class="space-y-6 py-2">
            <div class="bg-slate-900/50 border border-slate-800 rounded-2xl p-4">
                <h3 class="font-bold text-slate-100 text-lg mb-2">{selectedConv.title || 'Untitled Session'}</h3>
                <div class="flex flex-col gap-2">
                    <div class="flex items-center gap-2 text-xs text-slate-500">
                        <Terminal size={14} class="text-slate-600" />
                        <span class="font-mono">UUID: {selectedConv.id}</span>
                    </div>
                    <div class="flex items-center gap-2 text-xs text-slate-500">
                        <Calendar size={14} class="text-slate-600" />
                        <span>Registered on {formatDate(selectedConv.created_at)}</span>
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div class="bg-slate-950 border border-slate-800 rounded-2xl p-4">
                    <p class="text-[10px] font-black uppercase tracking-widest text-slate-600 mb-1">Current Usage</p>
                    <p class="text-xl font-mono font-bold text-blue-400">{formatTokens(selectedConv.cumulative_tokens)}</p>
                </div>
                <div class="bg-slate-950 border border-slate-800 rounded-2xl p-4">
                    <p class="text-[10px] font-black uppercase tracking-widest text-slate-600 mb-1">Status</p>
                    <div class="flex items-center gap-2">
                        <div class="w-2 h-2 rounded-full {getUsagePercentage(selectedConv.cumulative_tokens, selectedConv.max_token_usage) > 90 ? 'bg-rose-500 animate-pulse' : 'bg-emerald-500'}"></div>
                        <span class="text-xs font-bold text-slate-300">
                            {getUsagePercentage(selectedConv.cumulative_tokens, selectedConv.max_token_usage)}% Used
                        </span>
                    </div>
                </div>
            </div>

            <div class="space-y-3">
                <label for="max-tokens" class="text-[10px] font-black uppercase tracking-widest text-slate-500 flex items-center gap-2">
                    <Edit3 size={12} />
                    Maximum Token Allowance
                </label>
                <div class="relative">
                    <input 
                        id="max-tokens"
                        type="number" 
                        bind:value={editMaxTokens}
                        class="w-full bg-slate-950 border border-slate-800 rounded-2xl px-4 py-4 text-lg font-mono font-bold text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500/50 transition-all"
                    />
                    <div class="absolute right-4 top-1/2 -translate-y-1/2 text-[10px] font-black uppercase text-slate-600 tracking-widest pointer-events-none">
                        Tokens
                    </div>
                </div>
            </div>

            <!-- Dynamic Execution Boundaries (DEB) -->
            <div class="space-y-4 pt-2 border-t border-slate-800">
                <p class="text-[10px] font-black uppercase tracking-widest text-slate-400">Behavior Boundaries (DEB)</p>
                
                <!-- Interaction Gate -->
                <div class="space-y-2">
                    <div class="flex justify-between items-end">
                        <label class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Sociability</label>
                        <span class="text-[10px] font-mono font-black {getInteractionMode(editThresholds.interaction_gate).color}">
                            {getInteractionMode(editThresholds.interaction_gate).icon} {getInteractionMode(editThresholds.interaction_gate).label} ({editThresholds.interaction_gate.toFixed(2)})
                        </span>
                    </div>
                    <input 
                        type="range" 
                        min="0" max="1" step="0.01" 
                        bind:value={editThresholds.interaction_gate}
                        class="w-full h-1.5 bg-slate-900 rounded-lg appearance-none cursor-pointer accent-blue-500"
                        style="background: linear-gradient(to right, #3b82f6 0%, #3b82f6 {editThresholds.interaction_gate * 100}%, #0f172a {editThresholds.interaction_gate * 100}%, #0f172a 100%)"
                    />
                </div>

                <!-- Intent Classifier -->
                <div class="space-y-2">
                    <div class="flex justify-between items-end">
                        <label class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Confidence</label>
                        <span class="text-[10px] font-mono font-black {getIntentMode(editThresholds.intent_classification).color}">
                            {getIntentMode(editThresholds.intent_classification).icon} {getIntentMode(editThresholds.intent_classification).label} ({editThresholds.intent_classification.toFixed(2)})
                        </span>
                    </div>
                    <input 
                        type="range" 
                        min="0" max="1" step="0.01" 
                        bind:value={editThresholds.intent_classification}
                        class="w-full h-1.5 bg-slate-900 rounded-lg appearance-none cursor-pointer accent-purple-500"
                        style="background: linear-gradient(to right, #a855f7 0%, #a855f7 {editThresholds.intent_classification * 100}%, #0f172a {editThresholds.intent_classification * 100}%, #0f172a 100%)"
                    />
                </div>

                <!-- Guardrails -->
                <div class="space-y-2">
                    <div class="flex justify-between items-end">
                        <label class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Vigilance</label>
                        <span class="text-[10px] font-mono font-black {getGuardrailMode(editThresholds.guardrails).color}">
                            {getGuardrailMode(editThresholds.guardrails).icon} {getGuardrailMode(editThresholds.guardrails).label} ({editThresholds.guardrails.toFixed(2)})
                        </span>
                    </div>
                    <input 
                        type="range" 
                        min="0" max="1" step="0.01" 
                        bind:value={editThresholds.guardrails}
                        class="w-full h-1.5 bg-slate-900 rounded-lg appearance-none cursor-pointer accent-rose-500"
                        style="background: linear-gradient(to right, #f43f5e 0%, #f43f5e {editThresholds.guardrails * 100}%, #0f172a {editThresholds.guardrails * 100}%, #0f172a 100%)"
                    />
                </div>
            </div>
        </div>
    {/if}
{/snippet}

{#snippet detailFooter()}
    <div class="flex items-center justify-between gap-3 w-full">
        <button onclick={() => popupStore.closeLast()} class="px-6 py-2.5 text-xs font-bold text-slate-500 hover:text-slate-200 transition-colors">Cancel</button>
        <button onclick={handleUpdate} disabled={isSaving} class="flex items-center gap-2 px-8 py-2.5 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 rounded-xl text-white text-xs font-black uppercase tracking-widest shadow-lg shadow-blue-500/20 transition-all active:scale-95">
            {#if isSaving} <Loader2 size={14} class="animate-spin" /> Updating... {:else} <Save size={14} /> Save Changes {/if}
        </button>
    </div>
{/snippet}

<div class="flex flex-col h-full text-slate-200">
    <div class="flex-1 overflow-y-auto p-3 md:p-6 space-y-4 custom-scrollbar">
        {#if adminStore.convLoading && adminStore.conversations.length === 0}
            <div class="flex flex-col items-center justify-center py-24 gap-4">
                <Loader2 class="w-8 h-8 animate-spin text-blue-500" />
                <p class="text-[10px] font-black uppercase tracking-widest text-slate-600">Retrieving Metrics...</p>
            </div>
        {:else if adminStore.conversations.length === 0}
            <div class="text-center py-24 bg-slate-900/20 rounded-3xl border border-dashed border-slate-800 mx-2">
                <MessageSquare class="w-10 h-10 text-slate-800 mx-auto mb-4" />
                <p class="text-sm text-slate-500 px-4">No active conversations found.</p>
            </div>
        {:else}
            <div class="grid gap-3">
                {#each adminStore.conversations as conv (conv.id)}
                    <button onclick={() => openDetail(conv)} class="w-full text-left bg-slate-900/40 border border-slate-800/50 rounded-2xl p-4 md:p-5 hover:border-slate-700 hover:bg-slate-900/60 transition-all group active:scale-[0.99]">
                        <div class="flex flex-col gap-4">
                            <div class="flex flex-col sm:flex-row justify-between items-start gap-2">
                                <div class="flex-1 min-w-0 w-full">
                                    <h3 class="font-bold text-slate-100 truncate mb-1 group-hover:text-blue-400 transition-colors text-sm md:text-base">
                                        {conv.title || 'Untitled Session'}
                                    </h3>
                                    <div class="flex items-center gap-2 text-[9px] md:text-[10px] text-slate-500">
                                        <Calendar size={10} class="shrink-0" />
                                        <span class="truncate">{formatDate(conv.created_at)}</span>
                                    </div>
                                </div>
                                <div class="shrink-0">
                                    <span class="text-[8px] md:text-[9px] font-mono text-slate-600 uppercase tracking-widest bg-black/30 px-2 py-0.5 rounded border border-slate-800">
                                        {conv.id.substring(0, 8)}
                                    </span>
                                </div>
                            </div>

                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 md:gap-4">
                                <div class="bg-slate-950/50 border border-slate-800 rounded-xl p-3">
                                    <p class="text-[8px] md:text-[9px] font-black uppercase tracking-[0.15em] text-slate-600 mb-1.5 md:mb-2">Usage</p>
                                    <div class="flex items-baseline gap-1.5">
                                        <span class="text-base md:text-lg font-mono font-bold text-slate-200">{formatTokens(conv.cumulative_tokens)}</span>
                                        <span class="text-[9px] text-slate-500 uppercase font-black">Tkn</span>
                                    </div>
                                </div>
                                <div class="bg-slate-950/50 border border-slate-800 rounded-xl p-3">
                                    <p class="text-[8px] md:text-[9px] font-black uppercase tracking-[0.15em] text-slate-600 mb-1.5 md:mb-2">Limit</p>
                                    <div class="flex items-baseline gap-1.5">
                                        <span class="text-base md:text-lg font-mono font-bold text-slate-400">{formatTokens(conv.max_token_usage)}</span>
                                        <span class="text-[9px] text-slate-500 uppercase font-black">Tkn</span>
                                    </div>
                                </div>
                            </div>

                            <div class="space-y-1.5">
                                <div class="flex justify-between items-end">
                                    <p class="text-[8px] md:text-[9px] font-black uppercase tracking-widest text-slate-600">Efficiency</p>
                                    <p class="text-[9px] md:text-[10px] font-mono font-bold {getUsagePercentage(conv.cumulative_tokens, conv.max_token_usage) > 85 ? 'text-rose-400' : 'text-blue-400'}">
                                        {getUsagePercentage(conv.cumulative_tokens, conv.max_token_usage)}%
                                    </p>
                                </div>
                                <div class="h-1.5 w-full bg-slate-950 rounded-full overflow-hidden border border-slate-800/50">
                                    <div class="h-full transition-all duration-500 {getUsagePercentage(conv.cumulative_tokens, conv.max_token_usage) > 85 ? 'bg-rose-500 shadow-[0_0_10px_rgba(244,63,94,0.3)]' : 'bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.3)]'}" style="width: {getUsagePercentage(conv.cumulative_tokens, conv.max_token_usage)}%"></div>
                                </div>
                            </div>
                        </div>
                    </button>
                {/each}
            </div>
            
            {#if adminStore.hasMoreConvs}
                <div class="pt-4 flex justify-center">
                    <button onclick={() => adminStore.fetchConversations(true)} disabled={adminStore.convLoading} class="flex items-center gap-2 px-6 py-2 bg-slate-900 hover:bg-slate-800 border border-slate-800 rounded-xl text-[10px] font-black uppercase tracking-widest text-slate-400 hover:text-slate-200 transition-all active:scale-95 disabled:opacity-50">
                        {#if adminStore.convLoading} <Loader2 size={14} class="animate-spin" /> Loading... {:else} <ChevronDown size={14} /> Load More Threads {/if}
                    </button>
                </div>
            {/if}
        {/if}
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 4px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: #1e293b; border-radius: 10px; }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: #334155; }
</style>
