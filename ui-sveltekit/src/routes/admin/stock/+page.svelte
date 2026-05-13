<script lang="ts">
    import { onMount } from 'svelte';
    import { eventBus } from '$lib/utils';
    import { TrendingUp, TrendingDown, Minus, Activity, Clock, Wifi, WifiOff } from 'lucide-svelte';

    interface StockSignal {
        ticker: string;
        price: number;
        rsi: number;
        macd_hist: number;
        bb_upper: number;
        bb_lower: number;
        signal: 'BUY' | 'SELL' | 'NEUTRAL';
        timestamp: string;
        lastUpdated?: number; // timestamp in ms
    }

    let signals = $state<Record<string, StockSignal>>({});
    let isOnline = $state(false);
    let currentTime = $state(Date.now());

    // Sorted signals list (newest first based on timestamp, or just sorted by ticker)
    const sortedSignals = $derived(
        Object.values(signals).sort((a, b) => {
            const timeA = new Date(a.timestamp).getTime();
            const timeB = new Date(b.timestamp).getTime();
            return timeB - timeA;
        })
    );

    onMount(() => {
        const unsubscribeSignal = eventBus.subscribe('stock-signal', (data: StockSignal) => {
            signals[data.ticker] = {
                ...data,
                lastUpdated: Date.now()
            };
        });

        const unsubscribeStatus = eventBus.subscribe('gateway-status', (status: { online: boolean }) => {
            isOnline = status.online;
        });

        const timer = setInterval(() => {
            currentTime = Date.now();
        }, 1000);

        return () => {
            unsubscribeSignal();
            unsubscribeStatus();
            clearInterval(timer);
        };
    });

    function getSecondsAgo(timestamp: string | undefined) {
        if (!timestamp) return 'N/A';
        const seconds = Math.floor((currentTime - new Date(timestamp).getTime()) / 1000);
        return seconds < 0 ? 0 : seconds;
    }

    function getRsiColor(rsi: number) {
        if (rsi > 70) return 'text-red-400';
        if (rsi < 30) return 'text-green-400';
        return 'text-zinc-400';
    }

    function getSignalClass(signal: string) {
        switch (signal) {
            case 'BUY':
                return 'bg-green-500/10 text-green-500 border-green-500/50 animate-pulse';
            case 'SELL':
                return 'bg-red-500/10 text-red-500 border-red-500/50 animate-pulse';
            default:
                return 'bg-zinc-500/10 text-zinc-400 border-zinc-500/50';
        }
    }
</script>

{#snippet signalRow(signal: StockSignal)}
    <tr class="border-b border-zinc-800 hover:bg-zinc-900/50 transition-colors">
        <td class="py-4 px-6 font-bold text-zinc-100">{signal.ticker}</td>
        <td class="py-4 px-6 font-mono">
            {new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', maximumFractionDigits: 0 }).format(signal.price)}
        </td>
        <td class="py-4 px-6 font-mono {getRsiColor(signal.rsi)}">
            {signal.rsi.toFixed(2)}
        </td>
        <td class="py-4 px-6 font-mono {signal.macd_hist >= 0 ? 'text-green-400' : 'text-red-400'}">
            {signal.macd_hist.toFixed(2)}
        </td>
        <td class="py-4 px-6">
            <div class="flex flex-col text-[10px] text-zinc-500 font-mono">
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
            <div class="flex items-center justify-end gap-2 text-xs text-zinc-500">
                <Clock size={12} />
                {getSecondsAgo(signal.timestamp)}s ago
            </div>
        </td>
    </tr>
{/snippet}

<div class="p-6 max-w-7xl mx-auto h-full flex flex-col gap-6 overflow-hidden">
    <header class="flex items-center justify-between">
        <div class="flex items-center gap-3">
            <div class="p-2 bg-indigo-500/10 rounded-lg">
                <Activity class="text-indigo-500" size={24} />
            </div>
            <div>
                <h1 class="text-xl font-bold text-zinc-100">IDX Signals</h1>
                <p class="text-sm text-zinc-500">Technical Analysis Hub</p>
            </div>
        </div>

        <div class="flex items-center gap-4">
            <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-zinc-900 border border-zinc-800">
                {#if isOnline}
                    <Wifi size={14} class="text-green-500" />
                    <span class="text-xs font-medium text-green-500">Live</span>
                {:else}
                    <WifiOff size={14} class="text-red-500" />
                    <span class="text-xs font-medium text-red-500">Offline</span>
                {/if}
            </div>
        </div>
    </header>

    <div class="flex-1 overflow-auto rounded-xl border border-zinc-800 bg-zinc-950/50 backdrop-blur-sm">
        <table class="w-full text-left border-collapse">
            <thead class="sticky top-0 bg-zinc-950 z-10">
                <tr class="border-b border-zinc-800">
                    <th class="py-4 px-6 text-xs font-semibold text-zinc-500 uppercase tracking-wider">Ticker</th>
                    <th class="py-4 px-6 text-xs font-semibold text-zinc-500 uppercase tracking-wider">Price</th>
                    <th class="py-4 px-6 text-xs font-semibold text-zinc-500 uppercase tracking-wider">RSI (14)</th>
                    <th class="py-4 px-6 text-xs font-semibold text-zinc-500 uppercase tracking-wider">MACD Hist</th>
                    <th class="py-4 px-6 text-xs font-semibold text-zinc-500 uppercase tracking-wider">Bollinger</th>
                    <th class="py-4 px-6 text-xs font-semibold text-zinc-500 uppercase tracking-wider">Signal</th>
                    <th class="py-4 px-6 text-xs font-semibold text-zinc-500 uppercase tracking-wider text-right">Updated</th>
                </tr>
            </thead>
            <tbody>
                {#each sortedSignals as signal (signal.ticker)}
                    {@render signalRow(signal)}
                {:else}
                    <tr>
                        <td colspan="7" class="py-20 text-center text-zinc-500">
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
</div>

<style>
    :global(body) {
        background-color: #09090b;
    }
</style>