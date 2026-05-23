<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { eventBus } from '$lib/utils';
  import { 
    Factory, 
    History, 
    Play, 
    Zap, 
    CheckCircle2, 
    XCircle, 
    Loader2, 
    ChevronRight, 
    Terminal, 
    Code2,
    Settings2,
    Trash2,
    RefreshCw,
    Brain,
    Rocket,
    MonitorDot,
    FileCode,
    ChevronUp,
    ChevronDown,
    Maximize2,
    Minimize2,
    Beaker,
    Menu,
    X,
    ShieldAlert
  } from 'lucide-svelte';
  import { chatApi } from '$lib/api/client';
  import MonacoEditor from '$lib/components/MonacoEditor.svelte';
  import { popupStore } from '$lib/stores/popup.svelte';
  import BlueprintReviewPopUp from '$lib/components/BlueprintReviewPopUp.svelte';
  import SkillTesterPopUp from '$lib/components/SkillTesterPopUp.svelte';
  import { profileStore } from '$lib/stores/profile.svelte';

  let proposals = $state<any[]>([]);
  let selectedProposal = $state<any>(null);
  let liveLogs = $state<{time: string, log: string, step: string}[]>([]);
  let currentStep = $state("idle"); // idle, thinking, sandboxing, healing, success, failed
  let activeCodeOutput = $state("");
  let isLoadingProposals = $state(false);
  let logContainer: HTMLElement | undefined = $state();
  let logPanelHeight = $state(180); // Default collapsed height
  let isLogExpanded = $state(false);
  let isMobileNavOpen = $state(false);

  onMount(() => {
    reloadProposalsList();
    eventBus.subscribe('sse-evolution', handleEvolutionTelemetry);
  });

  onDestroy(() => {
    eventBus.unsubscribe('sse-evolution', handleEvolutionTelemetry);
  });

  function toggleLogPanel() {
    isLogExpanded = !isLogExpanded;
    logPanelHeight = isLogExpanded ? 400 : 180;
  }

  $effect(() => {
    if (liveLogs.length && logContainer) {
      logContainer.scrollTo({ top: logContainer.scrollHeight, behavior: 'smooth' });
    }
  });

  async function reloadProposalsList() {
    isLoadingProposals = true;
    try {
        const res = await chatApi.getProposals();
        if (res.data) {
            proposals = res.data;
        }
    } catch (e) {
        console.error("Failed to fetch proposals", e);
    } finally {
        isLoadingProposals = false;
    }
  }

  function handleEvolutionTelemetry(event: any) {
    if (!selectedProposal || event.slug !== selectedProposal.slug) return;

    if (event.log) {
        liveLogs = [...liveLogs, {
            time: new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
            log: event.log,
            step: event.step || currentStep
        }];
    }
    if (event.step) currentStep = event.step;
    if (event.code) activeCodeOutput = event.code;
    
    if (event.step === "success" || event.step === "failed") {
      reloadProposalsList();
    }
  }

  function selectProposal(item: any) {
    selectedProposal = item;
    isMobileNavOpen = false;
    liveLogs = [{
        time: new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
        log: `[MONITOR]: Attaching telemetry listener for [${item.slug}]...`,
        step: "monitor"
    }];
    currentStep = item.status;
    activeCodeOutput = item.compiled_code || "";

    if (item.status === 'pending') {
        popupStore.open({
            title: 'Blueprint Review',
            width: 'max-w-2xl',
            contentSnippet: blueprintReviewSnippet
        });
    }
  }

  async function launchBuild(slug: string) {
    try {
        const res = await chatApi.approveProposal(slug);
        if (res.data) {
            proposals = proposals.map(p => p.slug === slug ? { ...p, status: res.data.status } : p);
            const item = proposals.find(p => p.slug === slug);
            if (item) selectProposal(item);
        }
    } catch (e) {
        console.error("Build failed to launch", e);
    }
  }

  async function deployToProduction(slug: string) {
    liveLogs = [...liveLogs, {
        time: new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
        log: `[DEPLOYMENT]: Sending hot-patch request to gateway production runtime...`,
        step: "deploy"
    }];
    try {
        const res = await chatApi.deployProposal(slug);
        if (res.meta && res.meta.code === 200) {
          liveLogs = [...liveLogs, {
            time: new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
            log: `[SUCCESS]: Plugin hot-patched into live edge execution memory!`,
            step: "success"
          }];
          reloadProposalsList();
        } else {
          liveLogs = [...liveLogs, {
            time: new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
            log: `[DEPLOY ERROR]: Execution pass aborted.`,
            step: "failed"
          }];
        }
    } catch (e) {
        console.error("Deployment error", e);
    }
  }

  async function deleteProposal(slug: string) {
    if (!confirm("Are you sure you want to discard this blueprint?")) return;
    try {
        const res = await chatApi.deleteProposal(slug);
        if (res.meta && res.meta.code === 200) {
            if (selectedProposal?.slug === slug) selectedProposal = null;
            reloadProposalsList();
        }
    } catch (e) {
        console.error("Deletion failed", e);
    }
  }

  function launchTester() {
    if (!selectedProposal) return;
    popupStore.open({
        title: `Test Skill: ${selectedProposal.name}`,
        width: 'max-w-4xl',
        contentSnippet: skillTesterSnippet
    });
  }
</script>

<div class="daf-canvas w-full h-screen overflow-hidden flex flex-col bg-bg-main text-text-main font-sans relative">
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
        <Factory class="w-4 h-4 md:w-5 md:h-5 text-accent-emerald" />
      </div>
      <div>
        <h1 class="text-sm md:text-lg font-semibold tracking-tight text-white leading-none">Agent Factory</h1>
        <p class="text-[10px] md:text-xs text-text-muted mt-0.5 uppercase tracking-widest">Evolution Telemetry</p>
      </div>
    </div>

    <div class="flex items-center gap-4">
        <button onclick={reloadProposalsList} class="hidden sm:block p-2 hover:bg-border-main rounded-lg transition-colors text-text-muted hover:text-white">
            <RefreshCw class="w-4 h-4 {isLoadingProposals ? 'animate-spin' : ''}" />
        </button>
        <div class="flex items-center gap-2 bg-border-main px-2 md:px-3 py-1 md:py-1.5 rounded-full border border-accent-emerald/20 shadow-lg shadow-accent-emerald/5">
            <div class="w-1.5 h-1.5 md:w-2 md:h-2 bg-accent-emerald rounded-full animate-pulse shadow-[0_0_8px_#10b981]"></div>
            <span class="text-[8px] md:text-[10px] font-mono font-bold text-accent-emerald uppercase tracking-widest">Grid Live</span>
        </div>
    </div>
  </header>

  <div class="flex-1 flex overflow-hidden relative">
    <!-- Mobile Navigation Overlay -->
    {#if isMobileNavOpen}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div 
            class="fixed inset-0 bg-black/60 backdrop-blur-sm z-40 md:hidden animate-in fade-in duration-200"
            onclick={() => isMobileNavOpen = false}
        ></div>
    {/if}

    <!-- Sidebar: Proposal Queue -->
    <aside class="
        fixed inset-y-0 left-0 w-80 bg-[#111b21] border-r border-border-main z-50 transform transition-transform duration-300 ease-in-out md:relative md:translate-x-0 flex flex-col
        {isMobileNavOpen ? 'translate-x-0 shadow-2xl shadow-black/50' : '-translate-x-full'}
    ">
      <div class="p-4 border-b border-border-main flex items-center justify-between">
        <div class="flex items-center gap-2 text-[10px] font-bold text-text-muted uppercase tracking-wider">
          <History class="w-3.5 h-3.5" />
          Staging Blueprints
        </div>
        <div class="flex items-center gap-2">
            <span class="text-[10px] font-mono bg-border-main px-2 py-0.5 rounded text-text-muted">{proposals.length} Queue</span>
            <button onclick={() => isMobileNavOpen = false} class="md:hidden p-2 text-text-muted hover:text-white">
                <X class="w-4 h-4" />
            </button>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-3 flex flex-col gap-2 custom-scrollbar">
        {#each proposals as item}
          <button 
            onclick={() => selectProposal(item)}
            class="w-full text-left group bg-border-main/20 border p-4 rounded-xl transition-all duration-200 {selectedProposal?.slug === item.slug ? 'border-accent-emerald/40 bg-accent-emerald/5 ring-1 ring-accent-emerald/20' : 'border-border-main hover:border-border-main/60 hover:bg-border-main/30'}"
          >
            <div class="flex justify-between items-start mb-2">
              <h4 class="font-bold text-sm text-white truncate pr-2">{item.name}</h4>
              <span class="text-[9px] font-mono font-bold px-1.5 py-0.5 rounded uppercase border flex-shrink-0
                {item.status === 'pending' ? 'bg-amber-500/10 text-amber-500 border-amber-500/20' : ''}
                {item.status === 'approved' ? 'bg-blue-500/10 text-blue-400 border-blue-500/20 animate-pulse' : ''}
                {item.status === 'processing' ? 'bg-purple-500/10 text-purple-400 border-purple-500/20 animate-pulse' : ''}
                {item.status === 'ready' ? 'bg-emerald-500/10 text-accent-emerald border-accent-emerald/20' : ''}
                {item.status === 'deployed' ? 'bg-neutral-800 text-text-muted border-neutral-700' : ''}
                {item.status === 'failed' ? 'bg-rose-500/10 text-rose-500 border-rose-500/20' : ''}">
                {item.status}
              </span>
            </div>
            <p class="text-[11px] text-text-muted line-clamp-2 leading-relaxed mb-3">{item.description}</p>
            
            <div class="flex items-center justify-between pt-3 border-t border-border-main/30 opacity-0 group-hover:opacity-100 transition-opacity">
                <span class="text-[9px] font-mono text-text-muted/60">{item.slug}</span>
                <ChevronRight class="w-3 h-3 text-text-muted group-hover:text-accent-emerald transition-colors" />
            </div>
          </button>
        {:else}
            {#if !isLoadingProposals}
                <div class="flex flex-col items-center justify-center py-24 text-center px-6 opacity-30">
                    <Brain class="w-12 h-12 mb-4" />
                    <p class="text-sm font-bold italic">The factory floor is silent...</p>
                    <p class="text-[10px] mt-1 uppercase tracking-widest leading-relaxed">Ask Nomi to suggest a new skill to see blueprints appear here.</p>
                </div>
            {/if}
        {/each}
      </div>
    </aside>

    <!-- Factory Stage -->
    <main class="flex-1 overflow-y-auto p-4 md:p-6 bg-bg-main custom-scrollbar relative">
      {#if selectedProposal}
        <div class="max-w-6xl mx-auto space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
          
          <!-- Action Bar -->
          <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between bg-border-main/20 border border-border-main p-4 rounded-2xl backdrop-blur-sm shadow-xl gap-4">
            <div class="flex items-center gap-4">
                <div class="p-2.5 bg-bg-main rounded-xl border border-border-main">
                    <Settings2 class="w-5 h-5 text-accent-emerald" />
                </div>
                <div>
                    <h2 class="text-base md:text-lg font-bold text-white tracking-tight leading-tight">{selectedProposal.name}</h2>
                    <p class="text-[10px] md:text-xs text-text-muted font-mono">{selectedProposal.slug}</p>
                </div>
            </div>

            <div class="flex items-center gap-2 md:gap-3 w-full sm:w-auto">
                {#if profileStore.currentUser?.role === 'admin'}
                    <button onclick={() => deleteProposal(selectedProposal.slug)} class="p-2.5 hover:bg-rose-500/10 text-text-muted hover:text-rose-500 rounded-xl transition-all border border-transparent hover:border-rose-500/20" title="Discard Proposal">
                        <Trash2 class="w-4 h-4" />
                    </button>
                    
                    {#if selectedProposal.status === 'ready' || selectedProposal.status === 'deployed'}
                        <button 
                            onclick={launchTester}
                            class="flex-1 sm:flex-none px-4 py-2.5 bg-bg-main hover:bg-border-main text-text-muted hover:text-white border border-border-main rounded-xl text-[10px] md:text-xs font-bold uppercase tracking-widest transition-all flex items-center justify-center gap-2"
                        >
                            <Beaker class="w-3.5 h-3.5 text-amber-500" />
                            Test
                        </button>
                    {/if}

                    {#if selectedProposal.status === 'pending' || selectedProposal.status === 'failed'}
                        <button 
                            onclick={() => launchBuild(selectedProposal.slug)}
                            class="flex-1 sm:flex-none px-6 py-2.5 bg-accent-emerald hover:bg-accent-emerald/80 text-bg-main font-black rounded-xl text-[10px] md:text-xs uppercase tracking-widest transition-all flex items-center justify-center gap-2 shadow-lg shadow-accent-emerald/10"
                        >
                            <Play class="w-3.5 h-3.5 fill-current" />
                            Build
                        </button>
                    {:else if selectedProposal.status === 'ready'}
                        <button 
                            onclick={() => deployToProduction(selectedProposal.slug)}
                            class="flex-1 sm:flex-none px-6 py-2.5 bg-primary-blue hover:bg-primary-blue/80 text-white font-black rounded-xl text-[10px] md:text-xs uppercase tracking-widest transition-all flex items-center justify-center gap-2 shadow-lg shadow-primary-blue/20"
                        >
                            <Rocket class="w-3.5 h-3.5 fill-current" />
                            Deploy
                        </button>
                    {/if}
                {:else}
                    <div class="flex items-center gap-2 px-4 py-2 bg-border-main/40 rounded-xl border border-border-main/60 w-full justify-center">
                        <ShieldAlert class="w-3.5 h-3.5 text-amber-500" />
                        <span class="text-[10px] font-bold text-text-muted uppercase tracking-widest text-center italic">Read Only</span>
                    </div>
                {/if}
            </div>
          </div>

          <div class="flex flex-col h-[calc(100vh-250px)] sm:h-[calc(100vh-210px)] gap-4 min-h-[600px] md:min-h-0">
            
            <!-- Source Canvas (Top - Flex 1) -->
            <div class="flex-[2] flex flex-col bg-[#0b141a] border border-border-main rounded-2xl overflow-hidden shadow-2xl relative min-h-0">
              <div class="px-5 py-2.5 border-b border-border-main bg-border-main/40 flex items-center justify-between z-10 backdrop-blur-md">
                <div class="flex items-center gap-2 text-[10px] font-bold text-text-main uppercase tracking-widest">
                  <FileCode class="w-4 h-4 text-sky-400" />
                  Synthesized Source Canvas
                </div>
                <div class="hidden sm:block text-[9px] font-mono text-text-muted bg-black/40 px-2 py-0.5 rounded border border-white/5 uppercase tracking-tighter italic">TypeScript / Monaco</div>
              </div>

              <div class="flex-1 p-0 overflow-hidden relative bg-[#0d1117]">
                {#if activeCodeOutput}
                    <MonacoEditor 
                        bind:value={activeCodeOutput} 
                        language="typescript" 
                        readOnly={selectedProposal.status === 'deployed'} 
                    />
                {:else}
                    <div class="absolute inset-0 flex flex-col items-center justify-center text-center p-12 gap-4 opacity-10 grayscale select-none">
                        <Terminal class="w-16 h-16" />
                        <p class="text-xs uppercase tracking-[0.3em] font-black">Awaiting byte buffer...</p>
                    </div>
                {/if}
              </div>
            </div>

            <!-- Real-time Telemetry Monitor (Bottom - Fixed/Expandable) -->
            <div 
                class="flex-1 flex flex-col bg-[#0b141a] border border-border-main rounded-2xl overflow-hidden shadow-2xl relative transition-all duration-300 ease-in-out shrink-0 min-h-[150px]"
                style="height: {logPanelHeight}px"
            >
              <div class="px-5 py-2.5 border-b border-border-main bg-border-main/40 flex items-center justify-between z-10 backdrop-blur-md sticky top-0 cursor-default">
                <div class="flex items-center gap-4">
                    <div class="flex items-center gap-2 text-[10px] font-bold text-text-main uppercase tracking-widest">
                        <MonitorDot class="w-4 h-4 text-accent-emerald" />
                        Output Log
                    </div>
                    <div class="hidden sm:flex items-center gap-2">
                        <div class="w-1.5 h-1.5 bg-accent-emerald rounded-full animate-pulse"></div>
                        <span class="text-[9px] font-mono text-accent-emerald uppercase font-bold tracking-tighter">Status: {currentStep}</span>
                    </div>
                </div>
                
                <button 
                    onclick={toggleLogPanel}
                    class="p-1 hover:bg-white/10 rounded transition-colors text-text-muted hover:text-white"
                    title={isLogExpanded ? "Minimize Log" : "Expand Log"}
                >
                    {#if isLogExpanded}
                        <ChevronDown class="w-4 h-4" />
                    {:else}
                        <ChevronUp class="w-4 h-4" />
                    {/if}
                </button>
              </div>
              
              <div 
                bind:this={logContainer}
                class="flex-1 p-4 md:p-5 font-mono text-[10px] md:text-[11px] overflow-y-auto custom-scrollbar flex flex-col gap-1.5 bg-black/40"
              >
                {#each liveLogs as entry}
                  <div class="flex gap-3 md:gap-4 animate-in slide-in-from-left-1 duration-200 group border-b border-white/5 pb-1.5 last:border-0">
                    <span class="text-text-muted/30 flex-shrink-0 select-none w-12 md:w-14 font-bold">{entry.time}</span>
                    <div class="flex-1 flex flex-col gap-1">
                        <div class="flex items-center gap-2">
                            <span class="text-[7px] md:text-[8px] px-1 md:px-1.5 py-0.5 rounded uppercase font-black tracking-tighter
                                {entry.step === 'thinking' ? 'bg-blue-500/10 text-blue-400' : ''}
                                {entry.step === 'sandboxing' ? 'bg-purple-500/10 text-purple-400' : ''}
                                {entry.step === 'healing' ? 'bg-amber-500/10 text-amber-500' : ''}
                                {entry.step === 'success' ? 'bg-emerald-500/10 text-emerald-400' : ''}
                                {entry.step === 'failed' ? 'bg-rose-500/10 text-rose-400' : ''}
                                {entry.step === 'monitor' ? 'bg-neutral-800 text-neutral-500' : ''}
                                {entry.step === 'deploy' ? 'bg-primary-blue/10 text-primary-blue' : ''}
                            ">
                                {entry.step}
                            </span>
                        </div>
                        <span class="
                            {entry.log.includes('[SANDBOX TRACE ERROR]') || entry.log.includes('[DEPLOY ERROR]') ? 'text-rose-400 font-bold' : ''} 
                            {entry.log.includes('[VALIDATION SUCCESS]') || entry.log.includes('[SUCCESS]') ? 'text-accent-emerald font-bold' : ''}
                            {entry.log.includes('[FACTORY]') ? 'text-primary-blue underline underline-offset-4 decoration-primary-blue/30' : 'text-text-muted'}
                            leading-relaxed text-[11px]
                        ">{entry.log}</span>
                    </div>
                  </div>
                {/each}
                {#if currentStep === 'processing' || currentStep === 'thinking' || currentStep === 'sandboxing' || currentStep === 'healing'}
                    <div class="flex gap-4 py-2 opacity-50 italic animate-pulse border-t border-white/5 mt-2">
                        <span class="text-text-muted/30 flex-shrink-0 select-none w-14"></span>
                        <div class="flex items-center gap-2 text-accent-emerald text-[11px]">
                            <Loader2 class="w-3 h-3 animate-spin" />
                            <span>Executing {currentStep} cycle...</span>
                        </div>
                    </div>
                {/if}
              </div>
            </div>

          </div>
        </div>
      {:else}
        <div class="h-full flex flex-col items-center justify-center text-center p-8 animate-in fade-in duration-500">
          <div class="p-6 bg-border-main/20 rounded-full border border-border-main mb-6 relative shadow-2xl shadow-accent-emerald/5">
            <Factory class="w-10 md:w-12 h-10 md:h-12 text-text-muted opacity-40" />
            <div class="absolute inset-0 bg-accent-emerald/5 blur-2xl rounded-full"></div>
          </div>
          <h3 class="text-lg md:text-xl font-bold text-white tracking-tight mb-2 uppercase tracking-[0.2em]">DAF Command Center</h3>
          <p class="text-xs md:text-sm text-text-muted max-w-sm leading-relaxed font-medium italic">
            Select a staging profile from the queue to initialize telemetry streaming and source code synthesis monitors.
          </p>
        </div>
      {/if}
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

  .daf-canvas {
    background-color: #0f172a;
  }
</style>

{#snippet blueprintReviewSnippet()}
    {#if selectedProposal}
        <BlueprintReviewPopUp data={selectedProposal} />
    {/if}
{/snippet}

{#snippet skillTesterSnippet()}
    {#if selectedProposal}
        <SkillTesterPopUp 
            schema={{
                name: selectedProposal.slug,
                description: selectedProposal.description,
                parameters: selectedProposal.schema_json.parameters || selectedProposal.schema_json
            }} 
            scriptCode={selectedProposal.compiled_code}
        />
    {/if}
{/snippet}
