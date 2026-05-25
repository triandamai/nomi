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
    Play,
    Loader2,
    CheckCircle2,
    Info,
    ChevronRight,
    Brackets,
    Menu,
    X
  } from 'lucide-svelte';

  let { data } = $props();

  onMount(() => {
    srpStore.loadAvailablePlugins();
  });

  // 🌟 REACTIVE REFRESH: Trigger loadState whenever the URL slug changes
  $effect(() => {
    if (data.slug) {
        srpStore.loadState(data.slug);
    }
  });

  let pluginSlug = $derived(srpStore.state.slug || data.slug);
  let enrichedDescription = $derived(srpStore.state.enriched_description);
  let additionalRules = $derived(srpStore.state.additional_rules);
  let learnedPhrases = $derived(srpStore.state.learned_phrases);
  let isMobileNavOpen = $state(false);
</script>

<div class="flex flex-col h-screen bg-bg-main text-text-main overflow-hidden font-sans relative">
  <!-- Standard Header Alignment -->
  <header class="h-16 flex-shrink-0 flex items-center justify-between px-4 md:px-6 border-b border-border-main bg-bg-main/80 backdrop-blur-md z-30">
    <div class="flex items-center gap-3">
      <button 
        onclick={() => isMobileNavOpen = !isMobileNavOpen}
        class="md:hidden p-2 hover:bg-border-main rounded-lg text-text-muted transition-colors"
      >
        <Menu class="w-5 h-5" />
      </button>
      <div class="p-1.5 md:p-2 bg-accent-emerald/10 rounded-lg border border-accent-emerald/20">
        <Brain class="w-4 h-4 md:w-5 md:h-5 text-accent-emerald" />
      </div>
      <div>
        <h1 class="text-sm md:text-lg font-semibold tracking-tight text-text-main leading-none text-white">SRP Playground</h1>
        <p class="text-[10px] md:text-xs text-text-muted mt-0.5 uppercase tracking-widest">Tool reinforcement</p>
      </div>
    </div>

    <div class="flex items-center gap-2 bg-border-main px-2 md:px-3 py-1 md:py-1.5 rounded-full border border-accent-emerald/20 shadow-lg shadow-accent-emerald/5">
      <div class="w-1.5 h-1.5 md:w-2 md:h-2 bg-accent-emerald rounded-full animate-pulse shadow-[0_0_8px_#10b981]"></div>
      <span class="text-[8px] md:text-[10px] font-mono font-bold text-accent-emerald uppercase tracking-widest text-center italic text-white">Live Core</span>
    </div>
  </header>

  <!-- Main Content Grid -->
  <div class="flex-1 overflow-hidden flex flex-col md:flex-row relative">
    
    <!-- Mobile Navigation Overlay -->
    {#if isMobileNavOpen}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div 
            class="fixed inset-0 bg-black/60 backdrop-blur-sm z-40 md:hidden animate-in fade-in duration-200"
            onclick={() => isMobileNavOpen = false}
        ></div>
    {/if}

    <!-- Sidebar: Plugin Selection -->
    <aside class="
        fixed inset-y-0 left-0 w-64 bg-[#111b21] border-r border-border-main z-50 transform transition-transform duration-300 ease-in-out md:relative md:translate-x-0 flex flex-col
        {isMobileNavOpen ? 'translate-x-0 shadow-2xl shadow-black/50' : '-translate-x-full'}
    ">
      <div class="p-4 border-b border-border-main flex items-center justify-between md:block">
        <div class="flex items-center gap-2 text-[10px] font-bold text-text-muted uppercase tracking-wider md:mb-3">
          <History class="w-3.5 h-3.5" />
          Core Static Tools
        </div>
        <button onclick={() => isMobileNavOpen = false} class="md:hidden p-2 text-text-muted">
            <X class="w-4 h-4" />
        </button>
      </div>
      <div class="flex-1 overflow-y-auto p-2 flex flex-col gap-1 custom-scrollbar">
          {#each srpStore.availablePlugins as slug}
            <a 
              href="/dashboard/srp/{slug}" 
              onclick={() => isMobileNavOpen = false}
              class="group flex items-center justify-between px-3 py-2.5 rounded-lg text-sm transition-all {pluginSlug === slug ? 'bg-accent-emerald/10 text-accent-emerald border border-accent-emerald/30' : 'text-text-muted hover:bg-border-main hover:text-text-main border border-transparent'}"
            >
              <div class="flex items-center gap-2 truncate">
                <Cpu class="w-3.5 h-3.5 {pluginSlug === slug ? 'text-accent-emerald' : 'text-text-muted group-hover:text-text-main'}" />
                <span class="truncate font-mono text-xs">{slug}</span>
              </div>
              {#if pluginSlug === slug}
                <ChevronRight class="w-3 h-3 text-accent-emerald" />
              {/if}
            </a>
          {:else}
            <div class="flex items-center justify-center py-8">
              <Loader2 class="w-5 h-5 text-text-muted animate-spin" />
            </div>
          {/each}
      </div>
    </aside>

    <!-- Center Stage -->
    <main class="flex-1 overflow-y-auto p-4 md:p-6 space-y-6 custom-scrollbar bg-bg-main">
      
      <!-- Top Section: Enriched Description -->
      <section class="bg-border-main/20 border border-border-main rounded-xl overflow-hidden shadow-xl backdrop-blur-sm">
        <div class="px-5 py-3 border-b border-border-main bg-border-main/40 flex items-center justify-between">
          <div class="flex items-center gap-2 text-xs font-bold text-text-main uppercase tracking-wider">
            <Sparkles class="w-4 h-4 text-accent-emerald" />
            Optimized Instruction Manual
          </div>
          <div class="hidden sm:block text-[10px] font-mono text-text-muted italic">Auto-generated via user alignment passes</div>
        </div>
        <div class="p-4 md:p-5">
          {#if srpStore.isLoading}
            <div class="space-y-3">
              <div class="h-4 bg-border-main rounded w-3/4 animate-pulse"></div>
              <div class="h-4 bg-border-main rounded animate-pulse"></div>
            </div>
          {:else}
            <div class="p-4 bg-bg-main/80 border border-border-main rounded-lg text-sm text-text-main leading-relaxed font-mono relative group">
              <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                <Brackets class="w-4 h-4 text-text-muted" />
              </div>
              {enrichedDescription}
            </div>
          {/if}
        </div>
      </section>

      <!-- Mid Section: Two Column Split -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        
        <!-- Guardrails -->
        <section class="bg-border-main/20 border border-border-main rounded-xl overflow-hidden shadow-xl backdrop-blur-sm">
          <div class="px-5 py-3 border-b border-border-main bg-border-main/40 flex items-center justify-between">
            <div class="flex items-center gap-2 text-xs font-bold text-text-main uppercase tracking-wider">
              <ShieldCheck class="w-4 h-4 text-primary-blue" />
              Self-Generated Guardrails
            </div>
            <span class="text-[10px] font-mono text-text-muted">{additionalRules.length}/10 Capacity</span>
          </div>
          <div class="p-4 md:p-5 flex flex-col gap-3 min-h-[150px] md:min-h-[200px]">
            {#each additionalRules as rule}
              <div class="flex items-start gap-3 p-3 bg-bg-main/80 border border-border-main rounded-lg border-l-2 border-l-primary-blue shadow-sm">
                <CheckCircle2 class="w-4 h-4 text-primary-blue mt-0.5 flex-shrink-0" />
                <span class="text-xs font-mono text-text-main">{rule}</span>
              </div>
            {:else}
              <div class="flex-1 flex flex-col items-center justify-center text-text-muted/40">
                <Info class="w-8 h-8 mb-2 opacity-20" />
                <p class="text-xs italic">No operational guardrails cataloged yet</p>
              </div>
            {/each}
          </div>
        </section>

        <!-- Vocabulary Catchment -->
        <section class="bg-border-main/20 border border-border-main rounded-xl overflow-hidden shadow-xl backdrop-blur-sm">
          <div class="px-5 py-3 border-b border-border-main bg-border-main/40 flex items-center justify-between">
            <div class="flex items-center gap-2 text-xs font-bold text-text-main uppercase tracking-wider">
              <Zap class="w-4 h-4 text-amber-500" />
              Semantic Vocabulary
            </div>
            <span class="text-[10px] font-mono text-text-muted">{learnedPhrases.length}/25 Capacity</span>
          </div>
          <div class="p-4 md:p-5 flex flex-wrap gap-2 content-start min-h-[150px] md:min-h-[200px]">
            {#each learnedPhrases as phrase}
              <span class="text-[10px] font-mono bg-amber-500/5 text-amber-500 px-2.5 py-1.5 rounded border border-amber-500/20 hover:border-amber-500/40 transition-colors shadow-sm">
                {phrase}
              </span>
            {:else}
              <div class="w-full flex-1 flex flex-col items-center justify-center text-text-muted/40">
                <Info class="w-8 h-8 mb-2 opacity-20" />
                <p class="text-xs italic">No shorthand variants detected</p>
              </div>
            {/each}
          </div>
        </section>
      </div>

      <!-- Execution Simulator -->
      <section class="bg-border-main/20 border border-border-main rounded-xl overflow-hidden shadow-xl backdrop-blur-sm">
        <div class="px-5 py-3 border-b border-border-main bg-border-main/40 flex items-center">
          <div class="flex items-center gap-2 text-xs font-bold text-text-main uppercase tracking-wider">
            <Terminal class="w-4 h-4 text-accent-emerald" />
            Reinforcement & Learning Simulator
          </div>
        </div>
        <div class="p-4 md:p-6 grid grid-cols-1 lg:grid-cols-2 gap-6 md:gap-8">
          <div class="space-y-4">
            <div class="space-y-2">
              <label for="simulationInput" class="text-[10px] font-bold text-text-muted uppercase tracking-widest">Test Phrasing Shorthand</label>
              <textarea 
                id="simulationInput" 
                bind:value={srpStore.simulationInput} 
                class="w-full h-32 bg-bg-main/80 text-sm text-text-main p-4 rounded-xl border border-border-main focus:outline-none focus:border-accent-emerald/50 focus:ring-1 focus:ring-accent-emerald/20 transition-all resize-none font-mono placeholder:text-text-muted/40 shadow-inner" 
                placeholder="e.g., 'Nom log pengeluaran 50rb buat bakso...'"
              ></textarea>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <!-- Simulation Button -->
              <button 
                onclick={() => srpStore.runSimulation()} 
                disabled={srpStore.isSimulating || srpStore.isLearning || !srpStore.simulationInput} 
                class="py-3.5 bg-bg-main/60 hover:bg-bg-main border border-border-main hover:border-text-muted disabled:bg-border-main disabled:text-text-muted text-text-main font-bold rounded-xl text-[10px] uppercase tracking-widest transition-all flex items-center justify-center gap-2 shadow-sm active:scale-[0.98]"
              >
                {#if srpStore.isSimulating}
                  <Loader2 class="w-3.5 h-3.5 animate-spin" />
                  Simulating...
                {:else}
                  <Play class="w-3 h-3 fill-current text-text-muted" />
                  Run Simulation
                {/if}
              </button>

              <!-- Learn Button -->
              <button 
                onclick={() => srpStore.runDirectLearn()} 
                disabled={srpStore.isSimulating || srpStore.isLearning || !srpStore.simulationInput} 
                class="py-3.5 bg-gradient-to-r from-accent-emerald to-emerald-600 hover:from-accent-emerald/90 hover:to-emerald-600/90 text-bg-main font-black rounded-xl text-[10px] uppercase tracking-widest transition-all flex items-center justify-center gap-2 shadow-lg shadow-accent-emerald/10 active:scale-[0.98] disabled:from-border-main disabled:to-border-main disabled:text-text-muted disabled:shadow-none"
              >
                {#if srpStore.isLearning}
                  <Loader2 class="w-3.5 h-3.5 animate-spin" />
                  Learning Phrasing...
                {:else}
                  <Brain class="w-3.5 h-3.5 text-bg-main" />
                  Inject & Learn
                {/if}
              </button>
            </div>
          </div>

          <div class="space-y-2">
            <div class="text-[10px] font-bold text-text-muted uppercase tracking-widest">Alignment Outcome Trace</div>
            <div class="w-full h-[180px] bg-bg-main/80 p-4 rounded-xl border border-border-main overflow-y-auto text-xs font-mono text-text-muted custom-scrollbar whitespace-pre-wrap leading-relaxed shadow-inner">
              {#if srpStore.simulationOutput}
                <div class="text-accent-emerald mb-2 font-bold flex items-center gap-2">
                    <span class="w-1.5 h-1.5 bg-accent-emerald rounded-full animate-pulse"></span>
                    REINFORCEMENT LOG:
                </div>
                {srpStore.simulationOutput}
              {:else}
                <span class="text-text-muted/40 italic flex items-center gap-2">
                    <span class="w-1.5 h-1.5 bg-text-muted/20 rounded-full"></span>
                    // Trace results will populate following simulation pass...
                </span>
              {/if}
            </div>
          </div>
        </div>
      </section>
    </main>
  </div>
</div>

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
    height: 4px;
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
