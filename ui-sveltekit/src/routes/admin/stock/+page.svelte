<script lang="ts">
    import { stockStore, type StockSignal } from '$lib/stores/stock.svelte';
    import { Activity, Clock, TrendingUp, TrendingDown, Minus } from 'lucide-svelte';

    function getRsiColor(rsi: number) {
        if (rsi > 70) return 'text-red-400';
        if (rsi < 30) return 'text-green-400';
        return 'text-slate-400';
    }

    function getSignalClass(signal: string) {
        switch (signal) {
            case 'STRONG BUY':
                return 'bg-green-500/10 text-green-500 border-green-500/50 animate-pulse';
            case 'STRONG SELL':
                return 'bg-red-500/10 text-red-500 border-red-500/50 animate-pulse';
            default:
                return 'bg-slate-500/10 text-slate-400 border-slate-500/50';
        }
    }

    function formatCurrency(price: number) {
        return new Intl.NumberFormat('id-ID', { 
            style: 'currency', 
            currency: 'IDR', 
            maximumFractionDigits: 0 
        }).format(price);
    }
</script>

<div class="p-2 md:p-6 max-w-7xl mx-auto h-full flex flex-col gap-4 md:gap-6 overflow-hidden">
    <header class="flex items-center justify-between">
        <div class="flex items-center gap-3">
            <div class="p-2 bg-blue-500/10 rounded-lg">
                <Activity class="text-blue-500" size={24} />
            </div>
            <div>
                <h1 class="text-3xl font-black tracking-tight text-white">IDX Signals</h1>
                <p class="text-[10px] font-black uppercase tracking-widest text-slate-500">Technical Analysis Hub</p>
            </div>
        </div>
    </header>

    <!-- Desktop View: Table -->
    <div class="hidden md:block flex-1 overflow-auto no-scrollbar rounded-2xl border border-slate-800 bg-slate-950/50 backdrop-blur-sm">
        <table class="w-full text-left border-collapse">
            <thead class="sticky top-0 bg-slate-950 z-10">
                <tr class="border-b border-slate-800">
                    <th class="py-4 px-6 text-xs font-semibold text-slate-500 uppercase tracking-wider">Ticker</th>
                    <th class="py-4 px-6 text-xs font-semibold text-slate-500 uppercase tracking-wider">Price</th>
                    <th class="py-4 px-6 text-xs font-semibold text-slate-500 uppercase tracking-wider">RSI (14)</th>
                    <th class="py-4 px-6 text-xs font-semibold text-slate-500 uppercase tracking-wider">MACD Hist</th>
                    <th class="py-4 px-6 text-xs font-semibold text-slate-500 uppercase tracking-wider">Bollinger</th>
                    <th class="py-4 px-6 text-xs font-semibold text-slate-500 uppercase tracking-wider">Signal</th>
                    <th class="py-4 px-6 text-xs font-semibold text-slate-500 uppercase tracking-wider text-right">Updated</th>
                </tr>
            </thead>
            <tbody>
                {#each stockStore.sortedSignals as signal (signal.ticker)}
                    <tr class="border-b border-slate-800 hover:bg-slate-900/50 transition-colors">
                        <td class="py-4 px-6 font-bold text-slate-100">{signal.ticker}</td>
                        <td class="py-4 px-6 font-mono">
                            {formatCurrency(signal.price)}
                        </td>
                        <td class="py-4 px-6 font-mono {getRsiColor(signal.rsi)}">
                            {signal.rsi.toFixed(2)}
                        </td>
                        <td class="py-4 px-6 font-mono {signal.macd_hist >= 0 ? 'text-green-400' : 'text-red-400'}">
                            {signal.macd_hist.toFixed(2)}
                        </td>
                        <td class="py-4 px-6">
                            <div class="flex flex-col text-[10px] text-slate-500 font-mono">
                                <span>U: {signal.bb_upper.toFixed(0)}</span>
                                <span>L: {signal.bb_lower.toFixed(0)}</span>
                            </div>
                        </td>
                        <td class="py-4 px-6">
                            <span class="px-3 py-1 rounded-full text-xs font-bold border {getSignalClass(signal.signal)}">
                                {signal.signal}
                            </span>
                        </td>
                        <td class="py-4 px-6 text-right"> 
                            <div class="flex items-center justify-end gap-2 text-xs text-slate-500">
                                <Clock size={12} />
                                {stockStore.getSecondsAgo(signal.timestamp)}s ago
                            </div>
                        </td>
                    </tr>
                {:else}
                    <tr>
                        <td colspan="7" class="py-20 text-center text-slate-500">
                            <div class="flex flex-col items-center gap-2">
                                <Activity class="animate-pulse" size={40} />
                                <p>Waiting for incoming signals...</p>
                            </div>
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>

    <!-- Mobile View: Cards -->
    <div class="md:hidden flex-1 overflow-auto no-scrollbar flex flex-col gap-4">
        {#each stockStore.sortedSignals as signal (signal.ticker)}
            <div class="bg-slate-950/50 border border-slate-800 rounded-2xl p-4 flex flex-col gap-4">
                <div class="flex justify-between items-start">
                    <div>
                        <h2 class="text-lg font-bold text-slate-100">{signal.ticker}</h2>
                        <p class="text-sm font-mono text-slate-400">{formatCurrency(signal.price)}</p>
                    </div>
                    <span class="px-3 py-1 rounded-full text-[10px] font-bold border {getSignalClass(signal.signal)}">
                        {signal.signal}
                    </span>
                </div>

                <div class="grid grid-cols-2 gap-4 py-3 border-y border-slate-800/50">
                    <div class="flex flex-col">
                        <span class="text-[10px] uppercase text-slate-500">RSI (14)</span>
                        <span class="text-sm font-mono font-medium {getRsiColor(signal.rsi)}">
                            {signal.rsi.toFixed(2)}
                        </span>
                    </div>
                    <div class="flex flex-col">
                        <span class="text-[10px] uppercase text-slate-500">MACD Hist</span>
                        <span class="text-sm font-mono font-medium {signal.macd_hist >= 0 ? 'text-green-400' : 'text-red-400'}">
                            {signal.macd_hist.toFixed(2)}
                        </span>
                    </div>
                    <div class="flex flex-col col-span-2">
                        <span class="text-[10px] uppercase text-slate-500">Bollinger Bands</span>
                        <div class="flex gap-4 text-xs font-mono text-slate-400">
                            <span>Upper: {signal.bb_upper.toFixed(0)}</span>
                            <span>Lower: {signal.bb_lower.toFixed(0)}</span>
                        </div>
                    </div>
                </div>

                <div class="flex items-center justify-between text-xs text-slate-500">
                    <div class="flex items-center gap-1.5">
                        {#if signal.signal === 'STRONG BUY'}
                            <TrendingUp size={14} class="text-green-500" />
                            <span>Potential Entry</span>
                        {:else if signal.signal === 'STRONG SELL'}
                            <TrendingDown size={14} class="text-red-500" />
                            <span>Potential Exit</span>
                        {:else}
                            <Minus size={14} class="text-slate-500" />
                            <span>Accumulation</span>
                        {/if}
                    </div>
                    <div class="flex items-center gap-1">
                        <Clock size={12} />
                        {stockStore.getSecondsAgo(signal.timestamp)}s ago
                    </div>
                </div>
            </div>
        {:else}
            <div class="flex-1 flex flex-col items-center justify-center text-slate-500 gap-4">
                <Activity class="animate-pulse" size={48} />
                <p>Waiting for signals...</p>
            </div>
        {/each}
    </div>
</div>

<style>
    :global(body) {
        background-color: #0f172a;
    }

    .no-scrollbar::-webkit-scrollbar {
        display: none;
    }

    .no-scrollbar {
        -ms-overflow-style: none;
        scrollbar-width: none;
    }
</style>
