<script lang="ts">
    import { conversationStore } from '$lib/stores/conversation.svelte';
    import { formatTokenCount } from '$lib/utils';
    import { popupStore } from '$lib/stores/popup.svelte';
    import { apiFetch } from '$lib/api/client';
    import { Terminal, Calendar, Activity, Zap, Shield, MessageSquare, Loader2 } from 'lucide-svelte';

    const conv = $derived(conversationStore.activeConversation);
    const thresholds = $derived(conv?.gateway_thresholds || {
        interaction_gate: 0.6,
        intent_classification: 0.4,
        guardrails: 0.65
    });

    let subConvos = $state<any[]>([]);
    let isLoadingSub = $state(false);

    async function showSubConversations() {
        if (!conv) return;
        isLoadingSub = true;
        
        popupStore.open({
            title: 'Sub-Conversations (Channel Isolation)',
            width: 'max-w-xl',
            contentSnippet: subConvosSnippet
        });

        try {
            const res = await apiFetch<any[]>(`/conversations/${conv.id}/sub-conversations`);
            if (res.meta && res.meta.code === 200) {
                subConvos = res.data;
            } else {
                subConvos = [];
            }
        } catch (e) {
            console.error('Failed to load sub-conversations:', e);
            subConvos = [];
        } finally {
            isLoadingSub = false;
        }
    }

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

    function formatDate(dateStr: string | undefined | null) {
        if (!dateStr) return 'N/A';
        return new Date(dateStr).toLocaleString('id-ID', {
            day: '2-digit',
            month: 'short',
            year: 'numeric'
        });
    }
</script>

{#snippet subConvosSnippet()}
    <div class="space-y-4 py-2">
        {#if isLoadingSub}
            <div class="flex flex-col items-center justify-center py-20 gap-4">
                <Loader2 class="w-8 h-8 animate-spin text-blue-500" />
                <p class="text-[10px] font-black uppercase tracking-widest text-slate-500">Retrieving Sub-Chats...</p>
            </div>
        {:else if subConvos.length === 0}
            <div class="text-center py-16 bg-slate-900/20 rounded-3xl border border-dashed border-slate-800">
                <MessageSquare class="w-10 h-10 text-slate-800 mx-auto mb-4" />
                <p class="text-sm text-slate-500">No active sub-conversations found.</p>
                <p class="text-[10px] text-slate-650 mt-1.5 leading-relaxed max-w-xs mx-auto">
                    Sub-chats are spawned automatically to isolate communications across WhatsApp or Telegram bridges.
                </p>
            </div>
        {:else}
            <div class="grid gap-3">
                {#each subConvos as sub (sub.id)}
                    <div class="p-4 bg-slate-950 border border-slate-800 hover:border-slate-700 transition-colors rounded-2xl flex items-center justify-between gap-4">
                        <div class="min-w-0">
                            <h4 class="font-bold text-slate-200 text-sm truncate">{sub.title || 'Sub-Conversation'}</h4>
                            <div class="flex items-center gap-2 mt-1.5 text-[9.5px] font-mono text-slate-500">
                                <span>Type: <span class="text-slate-400 font-bold uppercase">{sub.conversation_type}</span></span>
                                <span>•</span>
                                <span>UUID: <span class="text-slate-400 font-bold">{sub.id.substring(0, 8)}...</span></span>
                            </div>
                        </div>
                        <div class="shrink-0 text-right">
                            <span class="text-xs font-mono font-bold text-blue-400">⚡ {formatTokenCount(sub.cumulative_tokens)} tokens</span>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
{/snippet}

<div class="space-y-6 py-2">
    {#if conv}
        <!-- Basic Info -->
        <div class="bg-slate-900/50 border border-slate-800 rounded-2xl p-4">
            <h3 class="font-bold text-slate-100 text-lg mb-2">{conv.name || 'Untitled Session'}</h3>
            <div class="flex flex-col gap-2">
                <div class="flex items-center gap-2 text-xs text-slate-500">
                    <Terminal size={14} class="text-slate-600" />
                    <span class="font-mono">UUID: {conv.id}</span>
                </div>
                <div class="flex items-center gap-2 text-xs text-slate-500">
                    <Calendar size={14} class="text-slate-600" />
                    <span>Created on {formatDate(conv.created_at)}</span>
                </div>
            </div>
        </div>

        <!-- Metrics & Actions -->
        <div class="space-y-3">
            <div class="grid grid-cols-2 gap-4">
                <div class="bg-slate-950 border border-slate-800 rounded-2xl p-4">
                    <p class="text-[10px] font-black uppercase tracking-widest text-slate-600 mb-1">Total Usage</p>
                    <p class="text-xl font-mono font-bold text-blue-400">{formatTokenCount(conv.cumulative_tokens)}</p>
                </div>
                <div class="bg-slate-950 border border-slate-800 rounded-2xl p-4">
                    <p class="text-[10px] font-black uppercase tracking-widest text-slate-600 mb-1">Limit</p>
                    <p class="text-xl font-mono font-bold text-slate-400">{formatTokenCount(conv.max_token_usage)}</p>
                </div>
            </div>

            <button 
                onclick={showSubConversations}
                class="w-full flex items-center justify-center gap-2 bg-gradient-to-r from-blue-600/10 to-indigo-600/10 hover:from-blue-600/20 hover:to-indigo-600/20 border border-blue-500/20 hover:border-blue-500/40 text-blue-300 font-black py-3 px-4 rounded-2xl text-[10px] uppercase tracking-widest transition-all active:scale-[0.98] shadow-lg shadow-blue-950/20"
            >
                <MessageSquare size={14} />
                Show Sub-Conversations
            </button>
        </div>

        <!-- DEB Display (Read Only) -->
        <div class="space-y-4 pt-4 border-t border-slate-800">
            <p class="text-[10px] font-black uppercase tracking-widest text-slate-400">Behavior Configuration (DEB)</p>
            
            <!-- Interaction Gate -->
            <div class="space-y-2 opacity-80">
                <div class="flex justify-between items-end">
                    <div class="flex items-center gap-2 text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                        <Activity size={12} />
                        Sociability
                    </div>
                    <span class="text-[10px] font-mono font-black {getInteractionMode(thresholds.interaction_gate).color}">
                        {getInteractionMode(thresholds.interaction_gate).icon} {getInteractionMode(thresholds.interaction_gate).label} ({thresholds.interaction_gate.toFixed(2)})
                    </span>
                </div>
                <div class="h-1.5 w-full bg-slate-900 rounded-full overflow-hidden">
                    <div class="h-full bg-blue-500 transition-all duration-500" style="width: {thresholds.interaction_gate * 100}%"></div>
                </div>
            </div>

            <!-- Intent Classifier -->
            <div class="space-y-2 opacity-80">
                <div class="flex justify-between items-end">
                    <div class="flex items-center gap-2 text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                        <Zap size={12} />
                        Confidence
                    </div>
                    <span class="text-[10px] font-mono font-black {getIntentMode(thresholds.intent_classification).color}">
                        {getIntentMode(thresholds.intent_classification).icon} {getIntentMode(thresholds.intent_classification).label} ({thresholds.intent_classification.toFixed(2)})
                    </span>
                </div>
                <div class="h-1.5 w-full bg-slate-900 rounded-full overflow-hidden">
                    <div class="h-full bg-purple-500 transition-all duration-500" style="width: {thresholds.intent_classification * 100}%"></div>
                </div>
            </div>

            <!-- Guardrails -->
            <div class="space-y-2 opacity-80">
                <div class="flex justify-between items-end">
                    <div class="flex items-center gap-2 text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                        <Shield size={12} />
                        Vigilance
                    </div>
                    <span class="text-[10px] font-mono font-black {getGuardrailMode(thresholds.guardrails).color}">
                        {getGuardrailMode(thresholds.guardrails).icon} {getGuardrailMode(thresholds.guardrails).label} ({thresholds.guardrails.toFixed(2)})
                    </span>
                </div>
                <div class="h-1.5 w-full bg-slate-900 rounded-full overflow-hidden">
                    <div class="h-full bg-rose-500 transition-all duration-500" style="width: {thresholds.guardrails * 100}%"></div>
                </div>
            </div>
        </div>

        <p class="text-[10px] text-slate-600 italic text-center pt-4">
            These parameters are optimized for this conversation's current context.
        </p>
    {/if}
</div>
