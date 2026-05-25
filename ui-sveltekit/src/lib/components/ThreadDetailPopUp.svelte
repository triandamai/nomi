<script lang="ts">
    import { conversationStore } from '$lib/stores/conversation.svelte';
    import { formatTokenCount } from '$lib/utils';
    import { Terminal, Calendar, Activity, Zap, Shield } from 'lucide-svelte';

    const conv = $derived(conversationStore.activeConversation);
    const thresholds = $derived(conv?.gateway_thresholds || {
        interaction_gate: 0.6,
        intent_classification: 0.4,
        guardrails: 0.65
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

    function formatDate(dateStr: string | undefined | null) {
        if (!dateStr) return 'N/A';
        return new Date(dateStr).toLocaleString('id-ID', {
            day: '2-digit',
            month: 'short',
            year: 'numeric'
        });
    }
</script>

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

        <!-- Metrics -->
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

        <!-- DEB Display (Read Only) -->
        <div class="space-y-4 pt-2 border-t border-slate-800">
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
