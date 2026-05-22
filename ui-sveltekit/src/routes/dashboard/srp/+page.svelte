<script lang="ts">
  import { srpStore } from "$lib/stores/srp.svelte";
  import { onMount } from "svelte";
  import { 
    Brain, 
    Cpu, 
    Zap, 
    ShieldCheck, 
    Terminal, 
    History, 
    Sparkles, 
    ArrowRight,
    Loader2,
    CheckCircle2,
    ChevronRight,
    Activity,
    CreditCard,
    Bell,
    Globe,
    FileText,
    Wrench
  } from 'lucide-svelte';

  onMount(() => {
    srpStore.loadAvailablePlugins();
  });

  const getIcon = (name: string) => {
    const n = name.toLowerCase();
    if (n.includes('finance') || n.includes('money')) return CreditCard;
    if (n.includes('health') || n.includes('vitality')) return Activity;
    if (n.includes('web') || n.includes('search')) return Globe;
    if (n.includes('remind') || n.includes('schedule') || n.includes('task')) return Bell;
    if (n.includes('doc') || n.includes('file') || n.includes('knowledge')) return FileText;
    return Wrench;
  };
</script>

<div class="flex flex-col h-screen bg-bg-main text-text-main overflow-hidden font-sans">
  <!-- Standard Header Alignment -->
  <header class="h-16 flex-shrink-0 flex items-center justify-between px-6 border-b border-border-main bg-bg-main/80 backdrop-blur-md">
    <div class="flex items-center gap-3">
      <div class="p-2 bg-accent-emerald/10 rounded-lg border border-accent-emerald/20">
        <Brain class="w-5 h-5 text-accent-emerald" />
      </div>
      <div>
        <h1 class="text-lg font-semibold tracking-tight text-text-main">SRP Registry</h1>
        <p class="text-xs text-text-muted">Self-Reinforcement Tool Intelligence</p>
      </div>
    </div>

    <div class="flex items-center gap-2 bg-border-main px-3 py-1.5 rounded-full border border-accent-emerald/20 shadow-lg shadow-accent-emerald/5">
      <div class="w-2 h-2 bg-accent-emerald rounded-full animate-pulse"></div>
      <span class="text-[10px] font-mono font-bold text-accent-emerald uppercase tracking-widest text-center">Reinforcement Engine Live</span>
    </div>
  </header>

  <!-- Main Grid View -->
  <main class="flex-1 overflow-y-auto p-8 custom-scrollbar bg-bg-main">
    <div class="max-w-6xl mx-auto">
      <div class="flex flex-col gap-8">
        
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-xl font-bold text-white tracking-tight">Active Learning Plugins</h2>
            <p class="text-sm text-text-muted mt-1">Core static tools currently evolving via user alignment passes.</p>
          </div>
        </div>

        {#if srpStore.isLoading && srpStore.availablePlugins.length === 0}
          <div class="flex flex-col items-center justify-center py-24 gap-4 opacity-50">
            <Loader2 class="w-8 h-8 animate-spin text-accent-emerald" />
            <p class="text-sm font-mono tracking-widest uppercase">Initializing Registry...</p>
          </div>
        {:else}
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {#each srpStore.availablePlugins as slug}
              {@const Icon = getIcon(slug)}
              <a 
                href="/dashboard/srp/{slug}"
                class="group bg-border-main/20 border border-border-main p-6 rounded-2xl hover:bg-border-main/40 hover:border-accent-emerald/30 transition-all duration-300 shadow-xl backdrop-blur-sm relative overflow-hidden"
              >
                <!-- Subtle Gradient Background -->
                <div class="absolute -right-4 -top-4 w-24 h-24 bg-accent-emerald/5 rounded-full blur-2xl group-hover:bg-accent-emerald/10 transition-colors"></div>
                
                <div class="flex flex-col h-full gap-4 relative z-10">
                  <div class="flex items-center justify-between">
                    <div class="p-3 bg-[#0b141a] rounded-xl border border-border-main group-hover:border-accent-emerald/20 transition-colors">
                      <Icon class="w-5 h-5 text-accent-emerald opacity-80 group-hover:opacity-100" />
                    </div>
                    <div class="flex items-center gap-1.5 px-2 py-1 rounded-md bg-emerald-500/5 text-[9px] font-black uppercase tracking-widest text-accent-emerald border border-emerald-500/10">
                      <Zap class="w-2.5 h-2.5 fill-current" />
                      Static
                    </div>
                  </div>

                  <div>
                    <h3 class="font-mono text-sm font-bold text-white group-hover:text-accent-emerald transition-colors">{slug}</h3>
                    <p class="text-[11px] text-text-muted mt-2 leading-relaxed line-clamp-2">
                      Core Nomi tool with autonomous reinforcement enabled. Click to audit learned rules and vocabulary shorthand.
                    </p>
                  </div>

                  <div class="mt-auto pt-4 border-t border-border-main/50 flex items-center justify-between">
                    <div class="flex items-center gap-2 text-[10px] font-bold text-text-muted">
                      <History class="w-3 h-3" />
                      Auto-Optimizing
                    </div>
                    <div class="flex items-center gap-1 text-[10px] font-bold text-accent-emerald opacity-0 group-hover:opacity-100 transition-opacity translate-x-2 group-hover:translate-x-0 transition-transform">
                      Enter SRP
                      <ArrowRight class="w-3 h-3" />
                    </div>
                  </div>
                </div>
              </a>
            {/each}
          </div>
        {/if}

        <!-- Info Card -->
        <div class="bg-primary-blue/5 border border-primary-blue/20 p-6 rounded-2xl flex gap-4 items-start">
          <div class="p-2 bg-primary-blue/10 rounded-lg text-primary-blue">
            <Sparkles class="w-5 h-5" />
          </div>
          <div>
            <h4 class="text-sm font-bold text-white">How Self-Reinforcement Works</h4>
            <p class="text-xs text-text-muted mt-2 leading-relaxed">
              Every time you use a core tool (like finance or reminders), Nomi's background engine analyzes the interaction. 
              If she finds a new way to describe a task or a better operational rule, she injects it into her own brain dynamically. 
              This registry allows you to monitor and simulate these learned behaviors.
            </p>
          </div>
        </div>

      </div>
    </div>
  </main>
</div>

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
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
