<script lang="ts">
    import { onMount } from 'svelte';
    import { Wallet, Tag, User, Receipt, AlertCircle, TrendingDown } from 'lucide-svelte';
    import { api } from '$lib/api/client';

    let { ref_id } = $props();

    let transaction = $state<any>(null);
    let loading = $state(true);
    let error = $state<string | null>(null);

    onMount(async () => {
        try {
            const res = await api.get<any>(`/money/history/${ref_id}`);
            if (res.data) {
                transaction = res.data;
            } else {
                error = res.meta?.message || "Transaction not found";
            }
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    });

    function formatCurrency(amount: number) {
        return new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', minimumFractionDigits: 0 }).format(amount);
    }
</script>

{#if loading}
    <div class="p-4 bg-slate-900/40 border border-slate-800 rounded-2xl animate-pulse flex items-center gap-3">
        <div class="w-10 h-10 bg-slate-800 rounded-lg"></div>
        <div class="flex-1 space-y-2">
            <div class="h-2 bg-slate-800 rounded w-2/3"></div>
            <div class="h-2 bg-slate-800 rounded w-1/3"></div>
        </div>
    </div>
{:else if error}
    <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-2xl flex items-center gap-3 text-red-400 text-xs italic">
        <AlertCircle class="w-4 h-4" />
        <span>Transaction details unavailable.</span>
    </div>
{:else if transaction}
    <div class="bg-slate-900/60 border border-indigo-500/30 rounded-2xl overflow-hidden shadow-xl backdrop-blur-md group hover:border-indigo-500/50 transition-all duration-300 max-w-sm">
        <div class="px-4 py-2 border-b border-white/5 bg-indigo-500/10 flex items-center justify-between">
            <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-indigo-400">
                <Wallet class="w-3 h-3" />
                Finance Entry
            </div>
            <div class="px-2 py-0.5 rounded-full bg-indigo-500/20 text-indigo-400 text-[8px] font-bold uppercase">
                {transaction.category}
            </div>
        </div>
        
        <div class="p-4 flex flex-col gap-4">
            <div>
                <h4 class="text-xs font-bold text-slate-400 uppercase tracking-widest mb-1">Merchant</h4>
                <p class="text-sm font-bold text-white truncate">{transaction.merchant_name}</p>
            </div>

            <div class="flex items-end justify-between gap-2">
                <div class="flex-1">
                    <h4 class="text-xs font-bold text-slate-400 uppercase tracking-widest mb-1">Total Amount</h4>
                    <p class="text-lg font-black text-white tracking-tight">
                        {formatCurrency(transaction.total_amount)}
                    </p>
                </div>
                <div class="p-2 bg-indigo-500/10 rounded-xl border border-indigo-500/20">
                    <TrendingDown class="w-5 h-5 text-indigo-400 opacity-80" />
                </div>
            </div>

            {#if transaction.items && transaction.items.length > 0}
                <div class="pt-3 border-t border-white/5">
                    <div class="flex items-center gap-1.5 text-[9px] text-slate-500 font-black uppercase tracking-[0.1em] mb-2">
                        <Receipt class="w-2.5 h-2.5" />
                        Line Items
                    </div>
                    <div class="space-y-1.5">
                        {#each transaction.items.slice(0, 3) as item}
                            <div class="flex justify-between text-[10px]">
                                <span class="text-slate-400 truncate flex-1">{item.name} (x{item.quantity})</span>
                                <span class="text-white font-mono">{formatCurrency(item.total_amount)}</span>
                            </div>
                        {/each}
                        {#if transaction.items.length > 3}
                            <p class="text-[8px] text-slate-500 italic">+{transaction.items.length - 3} more items...</p>
                        {/if}
                    </div>
                </div>
            {/if}
        </div>
    </div>
{/if}
