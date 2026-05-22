<script lang="ts">
    import { page } from '$app/state';
    import { 
        AlertCircle, 
        ArrowLeft, 
        Home, 
        Search, 
        ShieldAlert, 
        Terminal, 
        Ghost,
        Construction,
        RefreshCw
    } from 'lucide-svelte';
    import { goto } from '$app/navigation';

    // Extract error status and message
    const status = $derived(page.status);
    const message = $derived(page.error?.message || 'An unexpected technical error occurred.');

    const errorConfig = $derived.by(() => {
        if (status === 404) {
            return {
                title: "Reality Not Found",
                icon: Ghost,
                color: "text-amber-400",
                bg: "bg-amber-500/10",
                border: "border-amber-500/20",
                description: "The neural coordinates you requested do not exist in this workspace."
            };
        }
        if (status === 403) {
            return {
                title: "Access Restricted",
                icon: ShieldAlert,
                color: "text-rose-400",
                bg: "bg-rose-500/10",
                border: "border-rose-500/20",
                description: "Your credentials lack the authorization required for this capability domain."
            };
        }
        return {
            title: "System Fault Detected",
            icon: AlertCircle,
            color: "text-blue-400",
            bg: "bg-blue-500/10",
            border: "border-blue-500/20",
            description: "A critical exception has interrupted the orchestration loop."
        };
    });
</script>

<div class="min-h-screen bg-[#020617] flex flex-col items-center justify-center p-6 relative overflow-hidden">
    <!-- Background Neural Grid (Stylized) -->
    <div class="absolute inset-0 opacity-10 pointer-events-none">
        <div class="absolute inset-0" style="background-image: radial-gradient(#1e293b 1px, transparent 1px); background-size: 40px 40px;"></div>
    </div>
    
    <div class="absolute -top-40 -left-40 w-96 h-96 bg-blue-500/10 blur-[120px] rounded-full"></div>
    <div class="absolute -bottom-40 -right-40 w-96 h-96 bg-purple-500/10 blur-[120px] rounded-full"></div>

    <div class="max-w-xl w-full text-center space-y-12 relative z-10 animate-in fade-in zoom-in duration-700">
        <!-- Error Visual -->
        <div class="space-y-6">
            <div class="inline-flex items-center justify-center p-6 rounded-[2.5rem] {errorConfig.bg} border {errorConfig.border} shadow-2xl relative group">
                <div class="absolute inset-0 {errorConfig.bg} blur-2xl rounded-full opacity-0 group-hover:opacity-100 transition-opacity"></div>
                <svelte:component this={errorConfig.icon} class="w-16 h-16 {errorConfig.color} relative z-10" />
            </div>
            
            <div class="space-y-2">
                <div class="flex items-center justify-center gap-3">
                    <span class="px-2 py-0.5 bg-slate-900 border border-slate-800 rounded-md text-[10px] font-black uppercase tracking-[0.2em] text-slate-500">Error Code</span>
                    <span class="text-4xl font-black text-white tracking-tighter">{status}</span>
                </div>
                <h1 class="text-3xl md:text-5xl font-black text-white uppercase tracking-tight italic">
                    {errorConfig.title}
                </h1>
                <p class="text-slate-400 text-sm md:text-base leading-relaxed max-w-md mx-auto">
                    {errorConfig.description}
                </p>
            </div>
        </div>

        <!-- Technical Log -->
        <div class="bg-slate-950/80 border border-slate-900 rounded-2xl p-6 text-left space-y-3 backdrop-blur-sm">
            <div class="flex items-center gap-2 text-[10px] font-bold text-slate-600 uppercase tracking-widest border-b border-slate-900 pb-2">
                <Terminal class="w-3 h-3" />
                Kernel Exception Report
            </div>
            <p class="font-mono text-[11px] text-slate-400 break-all">
                <span class="text-rose-500/70">nomi_core_fault:</span> {message}
            </p>
        </div>

        <!-- Action Matrix -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <button 
                onclick={() => window.location.reload()}
                class="flex items-center justify-center gap-3 px-6 py-4 bg-slate-900 hover:bg-slate-800 border border-slate-800 rounded-2xl text-xs font-black uppercase tracking-widest text-slate-200 transition-all hover:scale-[1.02] active:scale-95"
            >
                <RefreshCw class="w-4 h-4 text-blue-400" />
                Retry Sync
            </button>
            
            <a 
                href="/"
                class="flex items-center justify-center gap-3 px-6 py-4 bg-blue-600 hover:bg-blue-500 rounded-2xl text-xs font-black uppercase tracking-widest text-white transition-all hover:scale-[1.02] active:scale-95 shadow-xl shadow-blue-950/20"
            >
                <Home class="w-4 h-4" />
                Return Home
            </a>
        </div>

        <p class="text-[9px] text-slate-700 font-bold uppercase tracking-[0.3em]">
            Experimental Multimodal OS — v2.0 Beta
        </p>
    </div>
</div>

<style>
    :global(body) {
        background-color: #020617;
        margin: 0;
        height: 100vh;
        overflow: hidden;
    }
</style>
