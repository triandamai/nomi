<script lang="ts">
    import { onMount } from 'svelte';
    import { Factory, Sparkles, ArrowRight, Cpu, AlertCircle, Loader2 } from 'lucide-svelte';
    import { api } from '$lib/api/client';
    import { goto } from '$app/navigation';

    let { ref_id } = $props();

    let proposal = $state<any>(null);
    let loading = $state(true);
    let error = $state<string | null>(null);

    onMount(async () => {
        try {
            const res = await api.get<any>(`/srp/proposals/${ref_id}`);
            if (res.data) {
                proposal = res.data;
            } else {
                error = res.meta?.message || "Proposal not found";
            }
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    });

    function getStatusColor(status: string) {
        switch (status) {
            case 'ready': return 'text-accent-emerald bg-accent-emerald/20';
            case 'processing': return 'text-amber-400 bg-amber-400/20';
            case 'failed': return 'text-red-400 bg-red-400/20';
            default: return 'text-slate-400 bg-slate-400/20';
        }
    }
</script>

{#if loading}
    <div class="p-5 bg-border-main/10 border border-border-main rounded-xl animate-pulse flex flex-col gap-4 max-w-sm">
        <div class="flex items-center justify-between">
            <div class="h-2 bg-slate-800 rounded w-1/3"></div>
            <div class="w-4 h-4 bg-slate-800 rounded-full"></div>
        </div>
        <div class="flex items-center gap-4">
            <div class="w-12 h-12 bg-slate-800 rounded-lg"></div>
            <div class="flex-1 space-y-2">
                <div class="h-3 bg-slate-800 rounded w-3/4"></div>
                <div class="h-2 bg-slate-800 rounded w-1/2"></div>
            </div>
        </div>
        <div class="h-10 bg-slate-800 rounded-lg w-full"></div>
    </div>
{:else if error}
    <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-xl flex items-center gap-3 text-red-400 text-xs italic">
        <AlertCircle class="w-4 h-4" />
        <span>Blueprint details unavailable.</span>
    </div>
{:else if proposal}
    <div class="bg-border-main/20 border border-border-main rounded-xl overflow-hidden shadow-2xl backdrop-blur-sm group/proposal hover:border-accent-emerald/40 transition-all duration-300 max-w-sm">
        <div class="px-5 py-3 border-b border-border-main bg-border-main/40 flex items-center justify-between">
            <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-accent-emerald">
                <Factory class="w-3.5 h-3.5" />
                Autonomous Blueprint Proposed
            </div>
            <Sparkles class="w-3.5 h-3.5 text-accent-emerald animate-pulse" />
        </div>
        <div class="p-5 flex flex-col gap-4">
            <div class="flex items-center gap-4">
                <div class="p-3 bg-bg-main rounded-lg border border-border-main group-hover/proposal:border-accent-emerald/20 transition-colors">
                    <Cpu class="w-6 h-6 text-accent-emerald opacity-80" />
                </div>
                <div class="flex-1 min-w-0">
                    <div class="flex items-center justify-between gap-2 mb-1">
                        <p class="text-[10px] font-bold text-text-muted uppercase tracking-widest truncate">Target Skill Handle</p>
                        <span class="px-1.5 py-0.5 rounded-md text-[8px] font-black uppercase {getStatusColor(proposal.status)}">
                            {proposal.status}
                        </span>
                    </div>
                    <h4 class="text-sm font-mono font-bold text-white truncate">{proposal.name}</h4>
                </div>
            </div>

            <p class="text-[11px] text-text-muted leading-relaxed line-clamp-2 italic">
                {proposal.description}
            </p>

            <button 
                onclick={() => goto(`/dashboard/srp/proposals`)}
                class="w-full py-3 bg-accent-emerald hover:bg-accent-emerald/80 text-bg-main font-black rounded-lg text-[10px] uppercase tracking-[0.2em] transition-all flex items-center justify-center gap-2 shadow-lg shadow-accent-emerald/10 active:scale-[0.98]"
            >
                <span>Open Factory Console</span>
                <ArrowRight class="w-3.5 h-3.5" />
            </button>
        </div>
    </div>
{/if}
