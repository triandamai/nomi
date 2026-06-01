<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '$lib/api/client';
    import { 
        Search, 
        Brain, 
        Loader2, 
        Activity, 
        ArrowLeft, 
        Calendar, 
        CheckCircle2, 
        AlertCircle, 
        Play, 
        Pause, 
        CornerDownRight 
    } from 'lucide-svelte';
    import { popupStore } from '$lib/stores/popup.svelte';
    import TaskCard from './TaskCard.svelte';

    interface TaskItem {
        id: string;
        conversation_id: string;
        title: string;
        global_goal: string;
        status: string;
        current_step_index: number;
        created_at: string;
    }

    let tasks = $state<TaskItem[]>([]);
    let searchQuery = $state('');
    let isLoading = $state(false);
    let selectedTask = $state<TaskItem | null>(null);

    let filteredTasks = $derived(
        tasks.filter(task => 
            task.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
            task.global_goal.toLowerCase().includes(searchQuery.toLowerCase()) ||
            task.status.toLowerCase().includes(searchQuery.toLowerCase())
        )
    );

    async function fetchTasks() {
        isLoading = true;
        try {
            const res = await api.get<TaskItem[]>('/tasks');
            if (res.data) {
                tasks = res.data;
            }
        } catch (e) {
            console.error('Failed to fetch tasks:', e);
        } finally {
            isLoading = false;
        }
    }

    function getStatusBadgeClass(status: string) {
        switch (status?.toLowerCase()) {
            case 'running': 
                return 'text-sky-400 bg-sky-500/10 border-sky-500/30';
            case 'paused_for_input': 
                return 'text-purple-400 bg-purple-500/15 border-purple-500/40 animate-pulse';
            case 'completed': 
                return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/30';
            case 'failed': 
                return 'text-rose-400 bg-rose-500/10 border-rose-500/30';
            default: 
                return 'text-slate-400 bg-slate-500/10 border-slate-500/30';
        }
    }

    function formatTime(dateStr: string) {
        return new Date(dateStr).toLocaleDateString(undefined, {
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    }

    onMount(() => {
        fetchTasks();
    });
</script>

<div class="text-slate-200 bg-transparent">
    {#if selectedTask}
        <!-- Detail Mode Header -->
        <div class="sticky top-0 bg-[#0f172a]/95 backdrop-blur-md border-b border-slate-800/60 p-4 -mx-6 -mt-6 z-10 flex items-center justify-between">
            <button 
                onclick={() => selectedTask = null} 
                class="flex items-center gap-2 px-3 py-1.5 rounded-xl bg-slate-900 border border-slate-800 text-slate-400 hover:text-white hover:border-slate-700 transition-all active:scale-95 text-xs font-bold"
            >
                <ArrowLeft size={14} />
                <span>Back to List</span>
            </button>
            <div class="flex items-center gap-2">
                <span class="text-[9px] font-black uppercase tracking-widest text-slate-500 font-mono">TASK ID</span>
                <span class="text-[10px] text-sky-400 font-mono font-bold bg-sky-500/5 px-2 py-0.5 rounded border border-sky-500/10">{selectedTask.id.substring(0, 8)}...</span>
            </div>
        </div>

        <!-- Task Timeline Detail Body -->
        <div class="space-y-6 pt-2">
            <!-- Header Card Summary -->
            <div class="p-5 bg-gradient-to-r from-slate-950/60 to-slate-900/40 border border-slate-800/50 rounded-2xl space-y-3">
                <div class="flex items-start justify-between gap-4">
                    <h3 class="text-base font-extrabold text-white tracking-wide">{selectedTask.title}</h3>
                    <span class="shrink-0 px-2.5 py-1 rounded-full text-[10px] font-black uppercase tracking-widest border font-mono {getStatusBadgeClass(selectedTask.status)}">
                        {selectedTask.status.replace(/_/g, ' ')}
                    </span>
                </div>
                <p class="text-xs text-slate-400 leading-relaxed font-medium">
                    {selectedTask.global_goal}
                </p>
                <div class="flex items-center gap-1.5 text-[10px] text-slate-500 pt-2 border-t border-slate-900">
                    <Calendar size={12} class="text-slate-600" />
                    <span>Auto-ignited: <span class="text-slate-400 font-bold">{formatTime(selectedTask.created_at)}</span></span>
                </div>
            </div>

            <!-- Nested HTO Interactive Visual Timeline -->
            <div class="border-t border-slate-900 pt-4">
                <p class="text-[10px] text-slate-500 uppercase font-black tracking-widest mb-4 flex items-center gap-2">
                    <Activity size={12} class="text-sky-500 animate-pulse" />
                    <span>Live Checklist & Timeline Logs</span>
                </p>
                <TaskCard ref_id={selectedTask.id} />
            </div>
        </div>
    {:else}
        <!-- List Mode Header -->
        <div class="sticky top-0 bg-[#0f172a]/95 backdrop-blur-md border-b border-slate-800/60 p-4 -mx-6 -mt-6 z-10 space-y-4">
            <div class="relative">
                <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
                <input
                    type="text"
                    bind:value={searchQuery}
                    placeholder="Search autonomous tasks by title, goal, or status..."
                    class="w-full bg-[#04060b] border border-slate-800/80 rounded-xl py-2.5 pl-10 pr-4 text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-sky-500/40 focus:border-sky-500/40 transition-all placeholder:text-slate-700"
                />
            </div>
        </div>

        <!-- Task List Body -->
        <div class="space-y-2 pt-2">
            {#if isLoading}
                <div class="flex flex-col items-center justify-center h-full space-y-3 opacity-60">
                    <Loader2 class="w-8 h-8 text-sky-500 animate-spin" />
                    <p class="text-slate-400 text-xs italic font-medium">Retrieving background agents...</p>
                </div>
            {:else if filteredTasks.length === 0}
                <div class="flex flex-col items-center justify-center h-full text-slate-500 space-y-2 opacity-50">
                    <Brain class="w-12 h-12 text-slate-800 mb-2" />
                    <p class="text-xs">No autonomous tasks found</p>
                </div>
            {:else}
                <div class="grid gap-2">
                    {#each filteredTasks as task}
                        <button 
                            onclick={() => selectedTask = task}
                            class="w-full text-left p-4 rounded-2xl bg-slate-900/40 border border-slate-800/50 hover:bg-slate-900/80 hover:border-sky-500/35 transition-all group flex flex-col gap-2.5"
                        >
                            <div class="flex items-start justify-between gap-4 w-full">
                                <div class="min-w-0 flex-1">
                                    <h4 class="text-xs font-bold text-slate-200 group-hover:text-sky-400 transition-colors uppercase tracking-wide truncate">
                                        {task.title}
                                    </h4>
                                    <p class="text-[11px] text-slate-400 mt-1 line-clamp-2 leading-relaxed">
                                        {task.global_goal}
                                    </p>
                                </div>
                                <span class="shrink-0 px-2 py-0.5 rounded text-[9px] font-black tracking-widest border font-mono {getStatusBadgeClass(task.status)}">
                                    {task.status.replace(/_/g, ' ')}
                                </span>
                            </div>

                            <div class="flex items-center justify-between w-full pt-3 border-t border-slate-950 text-[10px] text-slate-500">
                                <div class="flex items-center gap-1.5">
                                    <Calendar size={11} class="text-slate-600" />
                                    <span>{formatTime(task.created_at)}</span>
                                </div>
                                <div class="flex items-center gap-1 text-sky-500 font-black uppercase tracking-widest text-[9px] group-hover:translate-x-1 transition-transform">
                                    <span>View Details</span>
                                    <CornerDownRight size={10} />
                                </div>
                            </div>
                        </button>
                    {/each}
                </div>
            {/if}
        </div>

        <!-- Footer -->
        <div class="sticky bottom-0 bg-[#0f172a]/95 backdrop-blur-md border-t border-slate-800/60 p-4 -mx-6 -mb-6 z-10 flex items-center justify-between">
            <div class="flex items-center gap-2 text-[10px] text-slate-500 font-black uppercase tracking-widest">
                <Brain class="w-3.5 h-3.5 text-sky-500 animate-pulse" />
                <span>{filteredTasks.length} Autonomous Orchestrators</span>
            </div>
            <div class="text-[9px] text-slate-700 font-mono font-bold">
                HTO_V2_CLIENT
            </div>
        </div>
    {/if}
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
