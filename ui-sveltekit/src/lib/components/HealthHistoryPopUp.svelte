<script lang="ts">
    import { onMount } from 'svelte';
    import { healthStore } from '$lib/stores/health.svelte';
    import { Activity, Calendar, Heart, Moon, Footprints, Loader2, TrendingUp } from 'lucide-svelte';

    onMount(() => {
        healthStore.fetchHistory();
    });

    function handleDateChange() {
        healthStore.fetchHistory();
    }

    // Simple SVG Chart generator
    function generatePath(data: number[], width: number, height: number) {
        if (data.length < 2) return "";
        const max = Math.max(...data, 1);
        const step = width / (data.length - 1);
        return data.map((d, i) => {
            const x = i * step;
            const y = height - (d / max) * height;
            return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
        }).join(' ');
    }
</script>

<div class="flex flex-col h-full text-slate-200 bg-[#0f172a]/30">
    <!-- Date Selector Header -->
    <div class="p-4 border-b border-slate-800 space-y-4 bg-slate-900/50 backdrop-blur-md">
        <div class="flex items-center gap-4">
            <div class="flex-1 space-y-1">
                <label class="text-[10px] font-black uppercase tracking-widest text-slate-500 ml-1">Start Date</label>
                <div class="relative">
                    <Calendar class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-slate-500" />
                    <input 
                        type="date" 
                        bind:value={healthStore.startDate}
                        onchange={handleDateChange}
                        class="w-full bg-slate-950 border border-slate-800 rounded-xl pl-9 pr-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-blue-500/50 transition-all text-slate-300"
                    />
                </div>
            </div>
            <div class="flex-1 space-y-1">
                <label class="text-[10px] font-black uppercase tracking-widest text-slate-500 ml-1">End Date</label>
                <div class="relative">
                    <Calendar class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-slate-500" />
                    <input 
                        type="date" 
                        bind:value={healthStore.endDate}
                        onchange={handleDateChange}
                        class="w-full bg-slate-950 border border-slate-800 rounded-xl pl-9 pr-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-blue-500/50 transition-all text-slate-300"
                    />
                </div>
            </div>
        </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 overflow-y-auto p-4 custom-scrollbar space-y-6">
        {#if healthStore.loading && healthStore.history.length === 0}
            <div class="flex justify-center py-12">
                <Loader2 class="w-8 h-8 animate-spin text-blue-500" />
            </div>
        {:else if healthStore.history.length === 0}
            <div class="text-center py-12 bg-slate-900/20 rounded-3xl border border-dashed border-slate-800">
                <Activity class="w-12 h-12 text-slate-800 mx-auto mb-4" />
                <p class="text-sm text-slate-500">No biometrics synced for this period.</p>
            </div>
        {:else}
            <!-- Glanceable Cards -->
            <div class="grid grid-cols-3 gap-3">
                <div class="bg-gradient-to-br from-emerald-500/10 to-emerald-500/5 border border-emerald-500/20 p-4 rounded-3xl relative overflow-hidden group">
                    <Footprints class="w-8 h-8 text-emerald-500/20 absolute -right-1 -bottom-1 group-hover:scale-125 transition-transform" />
                    <p class="text-[10px] font-black uppercase tracking-widest text-emerald-500/60 mb-1">Steps</p>
                    <p class="text-xl font-black text-emerald-400 font-mono">{healthStore.stats.steps.toLocaleString()}</p>
                </div>
                <div class="bg-gradient-to-br from-rose-500/10 to-rose-500/5 border border-rose-500/20 p-4 rounded-3xl relative overflow-hidden group">
                    <Heart class="w-8 h-8 text-rose-500/20 absolute -right-1 -bottom-1 group-hover:scale-125 transition-transform" />
                    <p class="text-[10px] font-black uppercase tracking-widest text-rose-500/60 mb-1">Avg HR</p>
                    <p class="text-xl font-black text-rose-400 font-mono">{healthStore.stats.heart} <span class="text-[10px]">BPM</span></p>
                </div>
                <div class="bg-gradient-to-br from-indigo-500/10 to-indigo-500/5 border border-indigo-500/20 p-4 rounded-3xl relative overflow-hidden group">
                    <Moon class="w-8 h-8 text-indigo-500/20 absolute -right-1 -bottom-1 group-hover:scale-125 transition-transform" />
                    <p class="text-[10px] font-black uppercase tracking-widest text-indigo-500/60 mb-1">Sleep</p>
                    <p class="text-xl font-black text-indigo-400 font-mono">{healthStore.stats.sleep} <span class="text-[10px]">HRS</span></p>
                </div>
            </div>

            <!-- Charts Section -->
            <div class="space-y-4">
                <div class="bg-slate-900/50 border border-slate-800 rounded-3xl p-5">
                    <div class="flex justify-between items-center mb-6">
                        <div class="flex items-center gap-2">
                            <TrendingUp class="w-4 h-4 text-blue-400" />
                            <h4 class="text-xs font-black uppercase tracking-widest text-slate-400">Activity Trend</h4>
                        </div>
                    </div>
                    
                    <div class="h-32 w-full relative">
                        <svg class="w-full h-full" preserveAspectRatio="none">
                            <defs>
                                <linearGradient id="chartGradient" x1="0" y1="0" x2="0" y2="1">
                                    <stop offset="0%" stop-color="#3b82f6" stop-opacity="0.2" />
                                    <stop offset="100%" stop-color="#3b82f6" stop-opacity="0" />
                                </linearGradient>
                            </defs>
                            <!-- Grid Lines -->
                            {#each [0, 0.25, 0.5, 0.75, 1] as tick}
                                <line x1="0" y1={tick * 100 + "%"} x2="100%" y2={tick * 100 + "%"} stroke="#1e293b" stroke-width="1" />
                            {/each}
                            
                            <!-- Path -->
                            <path 
                                d={generatePath(healthStore.stepsData, 400, 128)} 
                                fill="none" 
                                stroke="#3b82f6" 
                                stroke-width="3" 
                                stroke-linecap="round" 
                                stroke-linejoin="round"
                                class="transition-all duration-500"
                            />
                        </svg>
                    </div>
                    <div class="flex justify-between mt-2 px-1">
                         <span class="text-[9px] font-mono text-slate-600 uppercase">{healthStore.startDate}</span>
                         <span class="text-[9px] font-mono text-slate-600 uppercase">{healthStore.endDate}</span>
                    </div>
                </div>
            </div>

            <!-- Detailed History List -->
            <div class="space-y-3">
                <h4 class="text-[10px] font-black uppercase tracking-widest text-slate-500 ml-1">Daily Breakdown</h4>
                <div class="space-y-2">
                    {#each healthStore.history.slice().reverse() as day}
                        <div class="bg-slate-900/40 border border-slate-800/50 rounded-2xl p-4 flex items-center justify-between hover:border-slate-700 transition-colors">
                            <div class="flex flex-col">
                                <span class="text-sm font-bold text-slate-200">{new Date(day.log_date).toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric' })}</span>
                                <span class="text-[10px] text-slate-500 font-mono uppercase tracking-tighter">Synced {new Date(day.updated_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                            </div>
                            <div class="flex gap-6">
                                <div class="text-right">
                                    <p class="text-[9px] font-black uppercase text-slate-600 tracking-tighter">Steps</p>
                                    <p class="text-xs font-bold text-emerald-500 font-mono">{(day.metrics.steps || 0).toLocaleString()}</p>
                                </div>
                                <div class="text-right">
                                    <p class="text-[9px] font-black uppercase text-slate-600 tracking-tighter">Heart</p>
                                    <p class="text-xs font-bold text-rose-500 font-mono">{day.metrics.avg_heart_rate || '--'} <span class="text-[8px]">BPM</span></p>
                                </div>
                                <div class="text-right">
                                    <p class="text-[9px] font-black uppercase text-slate-600 tracking-tighter">Sleep</p>
                                    <p class="text-xs font-bold text-indigo-500 font-mono">{day.metrics.sleep_hours || '--'} <span class="text-[8px]">H</span></p>
                                </div>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        {/if}
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
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: #334155;
    }
</style>
