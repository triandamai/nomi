<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { 
        Activity, 
        CheckCircle2, 
        Circle, 
        AlertCircle, 
        Clock, 
        ChevronDown, 
        ChevronUp, 
        Cpu,
        UserCheck,
        MessageSquareCode,
        Send
    } from 'lucide-svelte';
    import { api } from '$lib/api/client';
    import { eventBus } from '$lib/utils';

    let { ref_id, collapsed = false } = $props();

    let task = $state<any>(null);
    let loading = $state(true);
    let error = $state<string | null>(null);
    let expandedLogs = $state(false);
    let isCollapsed = $state(collapsed);
    let unsubscribeMqtt = $state<any>(null);

    async function fetchTaskTimeline() {
        try {
            const res = await api.get<any>(`/tasks/${ref_id}/timeline`);
            if (res.data) {
                task = res.data;
                error = null;
            } else {
                error = res.meta?.message || "Task not found";
            }
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        // 1. Load initial timeline snapshot once
        fetchTaskTimeline();

        // 2. Subscribe to real-time MQTT task updates.
        //    The MQTT signal is a lightweight ping (no logs, slim payload).
        //    Re-fetch the full timeline via HTTP on every ping so the UI
        //    always shows complete data regardless of MQTT packet size limits.
        unsubscribeMqtt = eventBus.subscribe('mqtt-task_update', (data: any) => {
            if (data && data.id === ref_id) {
                fetchTaskTimeline();
            }
        });
    });


    onDestroy(() => {
        if (unsubscribeMqtt) {
            unsubscribeMqtt();
        }
    });

    function getStatusColor(status: string) {
        switch (status?.toLowerCase()) {
            case 'running': return 'text-amber-400 bg-amber-500/10 border-amber-500/30';
            case 'paused_for_input': return 'text-purple-400 bg-purple-500/15 border-purple-500/40 animate-pulse';
            case 'completed': return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/30';
            case 'failed': return 'text-rose-400 bg-rose-500/10 border-rose-500/30';
            default: return 'text-slate-400 bg-slate-500/10 border-slate-500/30';
        }
    }

    function formatTime(dateStr: string) {
        return new Date(dateStr).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    }
</script>

{#if loading && !task}
    <div class="p-5 bg-slate-950/40 border border-slate-800 rounded-xl animate-pulse flex flex-col gap-3 max-w-md w-full">
        <div class="flex items-center gap-3">
            <div class="w-8 h-8 bg-slate-800 rounded-lg"></div>
            <div class="flex-1 space-y-2">
                <div class="h-3 bg-slate-800 rounded w-1/3"></div>
                <div class="h-2 bg-slate-800 rounded w-2/3"></div>
            </div>
        </div>
        <div class="h-20 bg-slate-900/60 rounded-lg"></div>
    </div>
{:else if error && !task}
    <div class="p-4 bg-rose-500/10 border border-rose-500/20 rounded-xl flex items-center gap-3 text-rose-400 text-xs italic max-w-md w-full">
        <AlertCircle class="w-4 h-4 shrink-0" />
        <span>Failed to load nomi workflow timeline: {error}</span>
    </div>
{:else if task}
    <div class="bg-slate-950/60 border border-slate-800 rounded-xl overflow-hidden shadow-2xl backdrop-blur-xl group hover:border-slate-700/80 transition-all duration-500 max-w-md w-full">
        <!-- Card Header -->
        <div class="px-5 py-3.5 border-b border-white/5 bg-slate-900/30 flex items-center justify-between gap-4">
            <div class="flex items-center gap-3 min-w-0">
                <div class="w-8 h-8 rounded-lg bg-purple-500/15 border border-purple-500/35 flex items-center justify-center text-purple-400 shrink-0 shadow-inner shadow-purple-500/10">
                    <Activity class="w-4.5 h-4.5" />
                </div>
                <div class="flex flex-col min-w-0">
                    <span class="text-xs font-black uppercase tracking-widest text-slate-400 flex items-center gap-1.5">
                        Nomi Workflow
                        {#if task.cumulative_tokens}
                            <span class="px-1.5 py-0.2 bg-white/5 border border-white/10 rounded text-slate-400 font-mono text-[9px] font-bold">
                                ⚡ {task.cumulative_tokens} tokens
                            </span>
                        {/if}
                    </span>
                    <h4 class="text-sm font-bold text-white truncate leading-snug">{task.title}</h4>
                </div>
            </div>
            <div class="px-2.5 py-0.5 rounded-full border text-[9px] font-black uppercase tracking-wider {getStatusColor(task.status)}">
                {task.status?.replace('_', ' ')}
            </div>
        </div>

        {#if isCollapsed}
            <button 
                onclick={() => isCollapsed = false}
                class="w-full py-2.5 bg-slate-900/40 hover:bg-slate-900/60 text-slate-400 hover:text-white border-t border-white/5 transition-all text-[10px] font-black uppercase tracking-widest flex items-center justify-center gap-2 group/btn cursor-pointer"
            >
                <span>Show Task Details</span>
                <ChevronDown class="w-3.5 h-3.5 group-hover/btn:translate-y-0.5 transition-transform" />
            </button>
        {:else}
            <div class="p-5 flex flex-col gap-4">
                <!-- Global Goal Description -->
                <div class="flex flex-col gap-1.5 p-3 rounded-lg bg-slate-900/40 border border-slate-800/40">
                    <span class="text-[9px] font-black uppercase tracking-widest text-slate-500">Global Objective</span>
                    <p class="text-xs font-medium text-slate-300 leading-relaxed">{task.global_goal}</p>
                </div>

                <!-- Checkpoint Checklists -->
                {#if task.checkpoints && Array.isArray(task.checkpoints)}
                    <div class="flex flex-col gap-2">
                        <span class="text-[9px] font-black uppercase tracking-widest text-slate-500 mb-1">Checkpoints Plan</span>
                        <div class="flex flex-col gap-2">
                            {#each task.checkpoints as cp}
                                {@const isActive = cp.step_index === task.current_step_index && task.status === 'running'}
                                {@const isCompleted = cp.status === 'completed' || cp.step_index < task.current_step_index}
                                {@const isFailed = cp.status === 'failed'}
                                <div class="flex items-start gap-3 p-2.5 rounded-xl border transition-all duration-300
                                    {isActive ? 'bg-amber-500/5 border-amber-500/30' : 
                                     isCompleted ? 'bg-emerald-500/5 border-emerald-500/20 opacity-70' : 
                                     isFailed ? 'bg-rose-500/5 border-rose-500/35' : 
                                     'bg-slate-900/10 border-slate-800/20 opacity-40'}">
                                    
                                    <div class="shrink-0 mt-0.5">
                                        {#if isCompleted}
                                            <CheckCircle2 class="w-4 h-4 text-emerald-400" />
                                        {:else if isFailed}
                                            <AlertCircle class="w-4 h-4 text-rose-400" />
                                        {:else if isActive}
                                            <div class="w-4 h-4 rounded-full border-2 border-amber-400 border-t-transparent animate-spin"></div>
                                        {:else}
                                            <Circle class="w-4 h-4 text-slate-600" />
                                        {/if}
                                    </div>
                                    <div class="flex-1 min-w-0">
                                        <span class="text-[8px] font-black tracking-widest uppercase block mb-0.5
                                            {isActive ? 'text-amber-400' : isCompleted ? 'text-emerald-400/80' : 'text-slate-500'}">
                                            Step {cp.step_index + 1}
                                        </span>
                                        <p class="text-xs font-bold leading-normal text-slate-200">{cp.action_objective}</p>
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </div>
                {/if}

                <!-- Collapsible Action Timeline Logs -->
                {#if task.logs && task.logs.length > 0}
                    <div class="border-t border-white/5 pt-4">
                        <button 
                            onclick={() => expandedLogs = !expandedLogs}
                            class="w-full flex items-center justify-between text-[10px] font-black uppercase tracking-widest text-slate-500 hover:text-slate-300 transition-colors cursor-pointer"
                        >
                            <div class="flex items-center gap-1.5">
                                <Cpu class="w-3.5 h-3.5 text-purple-400" />
                                <span>Action Timeline ({task.logs.length})</span>
                            </div>
                            {#if expandedLogs}
                                <ChevronUp class="w-3.5 h-3.5" />
                            {:else}
                                <ChevronDown class="w-3.5 h-3.5" />
                            {/if}
                        </button>

                        {#if expandedLogs}
                            <div class="mt-4 flex flex-col gap-3 border-l border-slate-800 ml-2 pl-4 py-1 animate-in fade-in slide-in-from-top-1 duration-300">
                                {#each task.logs as log}
                                    <div class="relative group/log flex flex-col gap-1">
                                        <!-- Event Node Marker -->
                                        <div class="absolute -left-[21.5px] top-1.5 w-2.5 h-2.5 rounded-full border-2 border-slate-950 shadow-md
                                            {log.event_type === 'step_start' ? 'bg-amber-400' :
                                             log.event_type === 'tool_execution' ? 'bg-blue-400' :
                                             log.event_type === 'human_response' ? 'bg-purple-400' :
                                             log.event_type === 'outbound_msg' ? 'bg-pink-400' :
                                             'bg-emerald-400'}">
                                        </div>

                                        <div class="flex items-center justify-between gap-3">
                                            <div class="flex items-center gap-1.5 text-[8px] font-black uppercase tracking-widest font-mono">
                                                {#if log.event_type === 'human_response'}
                                                    <UserCheck class="w-3 h-3 text-purple-400" />
                                                    <span class="text-purple-400">Human Input</span>
                                                {:else if log.event_type === 'tool_execution'}
                                                    <MessageSquareCode class="w-3 h-3 text-blue-400" />
                                                    <span class="text-blue-400">Tool Run</span>
                                                {:else if log.event_type === 'outbound_msg'}
                                                    <Send class="w-3 h-3 text-pink-400" />
                                                    <span class="text-pink-400">Outbound Msg</span>
                                                {:else}
                                                    <Cpu class="w-3 h-3 text-slate-500" />
                                                    <span class="text-slate-500">{log.event_type?.replace('_', ' ')}</span>
                                                {/if}
                                            </div>
                                            <span class="text-[8px] font-bold text-slate-600 tracking-wider font-mono flex items-center gap-1">
                                                <Clock class="w-2.5 h-2.5" />
                                                {formatTime(log.created_at)}
                                            </span>
                                        </div>
                                        <p class="text-[11px] text-slate-400 leading-normal line-clamp-3 overflow-hidden text-ellipsis" title={log.log_content}>
                                            {log.log_content}
                                        </p>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>
                {/if}
            </div>

            {#if collapsed}
                <button 
                    onclick={() => isCollapsed = true}
                    class="w-full py-2.5 bg-slate-900/40 hover:bg-slate-900/60 text-slate-400 hover:text-white border-t border-white/5 transition-all text-[10px] font-black uppercase tracking-widest flex items-center justify-center gap-2 group/btn cursor-pointer"
                >
                    <span>Hide Task Details</span>
                    <ChevronUp class="w-3.5 h-3.5 group-hover/btn:-translate-y-0.5 transition-transform" />
                </button>
            {/if}
        {/if}
    </div>
{/if}
