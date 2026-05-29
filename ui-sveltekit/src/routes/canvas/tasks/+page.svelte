<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '$lib/api/client';
    import { popupStore } from '$lib/stores/popup.svelte';
    import toast from 'svelte-french-toast';
    import { 
        ZoomIn, 
        ZoomOut, 
        Maximize2, 
        Search, 
        Brain, 
        RefreshCw,
        ArrowLeft,
        CheckCircle2,
        AlertCircle,
        XCircle,
        Play,
        Calendar,
        Activity,
        Sparkles
    } from 'lucide-svelte';
    import TaskCard from '$lib/components/TaskCard.svelte';

    interface TaskItem {
        id: string;
        conversation_id: string;
        title: string;
        global_goal: string;
        status: string;
        current_step_index: number;
        created_at: string;
    }

    // State Variables
    let windowWidth = $state(1024);
    let isMobile = $derived(windowWidth < 768);
    let tasks = $state<TaskItem[]>([]);
    let isLoading = $state(true);
    let isCancelling = $state(false);
    let searchQuery = $state('');

    // Zoom & Pan Canvas Transform State
    let zoom = $state(0.9);
    let panX = $state(50);
    let panY = $state(50);
    let isDragging = $state(false);
    let startX = $state(0);
    let startY = $state(0);

    // Dynamic Selected Task for Modal
    let selectedTaskId = $state<string | null>(null);
    let selectedTask = $derived(tasks.find(t => t.id === selectedTaskId) || null);

    // Zoom & Pan Mouse Listeners
    function handleMouseDown(e: MouseEvent) {
        if ((e.target as HTMLElement).closest('.node-card')) return;
        isDragging = true;
        startX = e.clientX - panX;
        startY = e.clientY - panY;
    }

    function handleMouseMove(e: MouseEvent) {
        if (!isDragging) return;
        panX = e.clientX - startX;
        panY = e.clientY - startY;
    }

    function handleMouseUp() {
        isDragging = false;
    }

    function handleWheel(e: WheelEvent) {
        e.preventDefault();
        const zoomFactor = 0.05;
        if (e.deltaY < 0) {
            zoom = Math.min(zoom + zoomFactor, 2.0);
        } else {
            zoom = Math.max(zoom - zoomFactor, 0.4);
        }
    }

    // Touch support for drag and pinch-to-zoom
    let isTouching = $state(false);
    let initialTouchDist = $state(0);
    let startZoom = $state(1.0);

    function handleTouchStart(e: TouchEvent) {
        if ((e.target as HTMLElement).closest('.node-card')) return;
        isTouching = true;
        
        if (e.touches.length === 1) {
            startX = e.touches[0].clientX - panX;
            startY = e.touches[0].clientY - panY;
        } else if (e.touches.length === 2) {
            initialTouchDist = Math.hypot(
                e.touches[0].clientX - e.touches[1].clientX,
                e.touches[0].clientY - e.touches[1].clientY
            );
            startZoom = zoom;
            startX = (e.touches[0].clientX + e.touches[1].clientX) / 2 - panX;
            startY = (e.touches[0].clientY + e.touches[1].clientY) / 2 - panY;
        }
    }

    function handleTouchMove(e: TouchEvent) {
        if (!isTouching) return;

        if (e.touches.length === 1) {
            panX = e.touches[0].clientX - startX;
            panY = e.touches[0].clientY - startY;
        } else if (e.touches.length === 2 && initialTouchDist > 0) {
            const dist = Math.hypot(
                e.touches[0].clientX - e.touches[1].clientX,
                e.touches[0].clientY - e.touches[1].clientY
            );
            zoom = Math.max(0.4, Math.min(startZoom * (dist / initialTouchDist), 2.0));
            const midX = (e.touches[0].clientX + e.touches[1].clientX) / 2;
            const midY = (e.touches[0].clientY + e.touches[1].clientY) / 2;
            panX = midX - startX;
            panY = midY - startY;
        }
    }

    function handleTouchEnd() {
        isTouching = false;
        initialTouchDist = 0;
    }

    function resetTransform() {
        if (isMobile) {
            zoom = 0.55;
            panX = 15;
            panY = 60;
        } else {
            zoom = 0.8;
            panX = 60;
            panY = 80;
        }
    }

    // API retrieval
    async function loadTasks() {
        isLoading = true;
        try {
            const res = await api.get<TaskItem[]>('/tasks');
            if (res.data) {
                tasks = res.data;
            }
        } catch (e) {
            console.error('Failed to fetch tasks:', e);
            toast.error('Failed to load background workflows');
        } finally {
            isLoading = false;
        }
    }

    // Cancel task execution
    async function cancelTask(taskId: string) {
        if (isCancelling) return;
        isCancelling = true;
        
        const loader = toast.loading('Aborting background orchestration workflow...');
        try {
            const res = await api.post<any>(`/tasks/${taskId}/cancel`);
            toast.success(res.data?.message || 'Task cancelled successfully!', { id: loader });
            popupStore.closeLast();
            selectedTaskId = null;
            await loadTasks();
        } catch (e: any) {
            console.error('Failed to cancel task:', e);
            toast.error(e.response?.data?.message || 'Failed to cancel task execution', { id: loader });
        } finally {
            isCancelling = false;
        }
    }

    // Node Type Helpers: Colors, Borders & Neon Glows
    function getNodeTheme(status: string) {
        switch (status?.toLowerCase()) {
            case 'running': 
                return {
                    border: 'border-sky-500/40 hover:border-sky-500 shadow-sky-500/10',
                    text: 'text-sky-400',
                    bg: 'bg-sky-500/5',
                    glow: 'shadow-[0_0_20px_rgba(59,130,246,0.15)]',
                    accent: '#3b82f6',
                    icon: Brain
                };
            case 'paused_for_input': 
            case 'pending':
                return {
                    border: 'border-purple-500/40 hover:border-purple-500 shadow-purple-500/10',
                    text: 'text-purple-400',
                    bg: 'bg-purple-500/5',
                    glow: 'shadow-[0_0_20px_rgba(168,85,247,0.15)] animate-pulse',
                    accent: '#a855f7',
                    icon: Brain
                };
            case 'completed': 
                return {
                    border: 'border-emerald-500/40 hover:border-emerald-500 shadow-emerald-500/10',
                    text: 'text-emerald-400',
                    bg: 'bg-emerald-500/5',
                    glow: 'shadow-[0_0_20px_rgba(16,185,129,0.15)]',
                    accent: '#10b981',
                    icon: CheckCircle2
                };
            case 'failed': 
            case 'cancelled':
                return {
                    border: 'border-rose-500/30 hover:border-rose-500/50 shadow-rose-500/5',
                    text: 'text-rose-400/80',
                    bg: 'bg-rose-500/5',
                    glow: 'opacity-75',
                    accent: '#f43f5e',
                    icon: AlertCircle
                };
            default: 
                return {
                    border: 'border-slate-800 hover:border-slate-600',
                    text: 'text-slate-400',
                    bg: 'bg-slate-500/5',
                    glow: '',
                    accent: '#64748b',
                    icon: Brain
                };
        }
    }

    // Dynamic Svelte coordinates mapping based on task status lanes
    let visibleFilteredNodes = $derived.by(() => {
        const matching = tasks.filter(task => {
            if (!searchQuery) return true;
            return task.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
                   task.global_goal.toLowerCase().includes(searchQuery.toLowerCase()) ||
                   task.status.toLowerCase().includes(searchQuery.toLowerCase());
        });

        // Lane groupings
        const pending = matching.filter(t => t.status === 'pending' || t.status === 'paused_for_input');
        const running = matching.filter(t => t.status === 'running');
        const completed = matching.filter(t => t.status === 'completed');
        const failed = matching.filter(t => t.status === 'failed');

        const nodesList: any[] = [];
        const rowGap = isMobile ? 120 : 130;

        // Position nodes vertically in status columns
        pending.forEach((task, index) => {
            nodesList.push({
                ...task,
                x: isMobile ? 50 : 80,
                y: isMobile ? 180 + index * rowGap : 160 + index * rowGap
            });
        });

        running.forEach((task, index) => {
            nodesList.push({
                ...task,
                x: isMobile ? 50 : 420,
                y: isMobile ? 180 + (pending.length + index) * rowGap : 160 + index * rowGap
            });
        });

        completed.forEach((task, index) => {
            nodesList.push({
                ...task,
                x: isMobile ? 50 : 760,
                y: isMobile ? 180 + (pending.length + running.length + index) * rowGap : 160 + index * rowGap
            });
        });

        failed.forEach((task, index) => {
            nodesList.push({
                ...task,
                x: isMobile ? 50 : 1100,
                y: isMobile ? 180 + (pending.length + running.length + completed.length + index) * rowGap : 160 + index * rowGap
            });
        });

        return nodesList;
    });

    // Central virtual Orchestrator Node position helper (between columns 2 & 3)
    const orchestratorNode = $derived({
        id: 'central-orchestrator-node',
        label: 'Agentic Core',
        subtitle: 'Tokio orchestrator',
        status: 'active',
        x: isMobile ? 50 : 590,
        y: isMobile ? 40 : 40
    });

    // Bezier line path calculation
    function calculateBezierPath(source: any, target: any) {
        if (!source || !target) return '';
        
        // Output from bottom center of Central Orchestrator
        const x1 = source.x + 125;
        const y1 = source.y + 70;
        
        // Input at top center of target task card
        const x2 = target.x + 125;
        const y2 = target.y;

        const controlOffset = Math.abs(y2 - y1) * 0.45;
        return `M ${x1} ${y1} C ${x1} ${y1 + controlOffset}, ${x2} ${y2 - controlOffset}, ${x2} ${y2}`;
    }

    function formatTime(dateStr: string) {
        return new Date(dateStr).toLocaleDateString(undefined, {
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    }

    function handleNodeClick(node: TaskItem) {
        selectedTaskId = node.id;
        popupStore.open({
            title: node.title,
            width: 'max-w-3xl h-[100dvh]',
            contentSnippet: taskTimelineModalSnippet
        });
    }

    onMount(() => {
        windowWidth = window.innerWidth;
        loadTasks();
        resetTransform();
    });
</script>

<!-- Timeline and Details popup snippet -->
{#snippet taskTimelineModalSnippet()}
    {#if selectedTask}
        <div class="h-full flex flex-col">
            <!-- Timeline Log timeline checklist body -->
            <div class="flex-1 overflow-y-auto custom-scrollbar p-6 space-y-6">
                <!-- Header Summary Panel -->
                <div class="p-5 bg-gradient-to-r from-slate-950/60 to-slate-900/40 border border-slate-800/50 rounded-2xl space-y-3">
                    <div class="flex items-start justify-between gap-4">
                        <h3 class="text-base font-extrabold text-white tracking-wide">{selectedTask.title}</h3>
                        <span class="shrink-0 px-2.5 py-1 rounded-full text-[10px] font-black uppercase tracking-widest border font-mono {getNodeTheme(selectedTask.status).text} bg-slate-950/60 border-slate-800/35">
                            {selectedTask.status.replace(/_/g, ' ')}
                        </span>
                    </div>
                    <p class="text-xs text-slate-400 leading-relaxed font-medium">
                        {selectedTask.global_goal}
                    </p>
                    <div class="flex items-center gap-1.5 text-[10px] text-slate-500 pt-2 border-t border-slate-900">
                        <Calendar size={12} class="text-slate-600" />
                        <span>Created: <span class="text-slate-400 font-bold">{formatTime(selectedTask.created_at)}</span></span>
                    </div>
                </div>

                <!-- Sub-Steps Timeline details -->
                <div class="border-t border-slate-900 pt-4">
                    <p class="text-[10px] text-slate-500 uppercase font-black tracking-widest mb-4 flex items-center gap-2">
                        <Activity size={12} class="text-sky-500 animate-pulse" />
                        <span>Live Execution Steps & Telemetry</span>
                    </p>
                    <TaskCard ref_id={selectedTask.id} />
                </div>
            </div>

            <!-- Action panel footer with direct cancellation endpoint trigger -->
            {#if selectedTask.status === 'running' || selectedTask.status === 'paused_for_input' || selectedTask.status === 'pending'}
                <div class="p-4 border-t border-slate-800/40 bg-slate-950/80 backdrop-blur-md flex items-center gap-3">
                    <button 
                        onclick={() => cancelTask(selectedTask!.id)}
                        disabled={isCancelling}
                        class="w-full py-3.5 bg-rose-500/10 hover:bg-rose-500 hover:text-white text-rose-400 border border-rose-500/20 hover:border-rose-500 disabled:opacity-50 transition-all font-black uppercase tracking-widest text-[10px] md:text-xs rounded-xl flex items-center justify-center gap-2"
                    >
                        <XCircle size={16} class={isCancelling ? 'animate-spin' : ''} />
                        <span>Abort & Terminate Workflow</span>
                    </button>
                </div>
            {/if}
        </div>
    {/if}
{/snippet}

<svelte:window bind:innerWidth={windowWidth} onresize={resetTransform} />

<!-- Full Screen Canvas Wrapper -->
<div class="relative w-full h-[100dvh] bg-transparent text-slate-100 overflow-hidden font-sans select-none">
    
    <!-- Dynamic Pattern Grid responsive background dot mapping -->
    <div 
        class="absolute inset-0 pointer-events-none transition-all duration-75"
        style="
            background-image: radial-gradient(var(--slate-800) 1.5px, transparent 1.5px);
            background-size: {24 * zoom}px {24 * zoom}px;
            background-position: {panX}px {panY}px;
            opacity: 0.85;
        "
    ></div>

    <!-- Header Floating Deck -->
    <div class="absolute top-6 left-6 right-6 z-20 flex flex-col sm:flex-row gap-4 items-center justify-between pointer-events-none">
        
        <!-- Left Title Navigation anchor -->
        <div class="flex items-center gap-3 bg-slate-950/85 backdrop-blur-md border border-slate-800/80 px-5 py-3 rounded-2xl shadow-xl pointer-events-auto w-full sm:w-auto justify-between sm:justify-start">
            <div class="flex items-center gap-3">
                <button onclick={() => window.history.back()} class="p-1.5 rounded-xl hover:bg-slate-900 text-slate-400 hover:text-white transition-colors">
                    <ArrowLeft size={16} />
                </button>
                <div class="h-4 w-px bg-slate-800"></div>
                <div class="flex items-center gap-2">
                    <Brain class="w-4 h-4 text-sky-500 animate-pulse" />
                    <span class="text-xs font-black uppercase tracking-[0.2em] text-slate-300">Orchestrator Canvas</span>
                </div>
            </div>
        </div>

        <!-- Right controller spotlight search -->
        <div class="flex items-center gap-3 bg-slate-950/85 backdrop-blur-md border border-slate-800/80 px-4 py-2.5 rounded-2xl shadow-xl pointer-events-auto w-full sm:w-[280px]">
            <Search size={16} class="text-slate-500 shrink-0" />
            <input 
                type="text" 
                bind:value={searchQuery}
                placeholder="Search workflows..." 
                class="bg-transparent border-none text-xs text-slate-200 outline-none w-full placeholder:text-slate-700 font-medium"
            />
        </div>
    </div>

    <!-- Bottom Controls floating deck -->
    <div class="absolute bottom-6 right-6 z-20 flex items-center gap-2 bg-slate-950/85 backdrop-blur-md border border-slate-800/80 p-2 rounded-2xl shadow-xl">
        <button 
            onclick={() => zoom = Math.min(zoom + 0.1, 2.0)} 
            class="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-900 transition-colors"
            title="Zoom In"
        >
            <ZoomIn size={16} />
        </button>
        <button 
            onclick={() => zoom = Math.max(zoom - 0.1, 0.4)} 
            class="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-900 transition-colors"
            title="Zoom Out"
        >
            <ZoomOut size={16} />
        </button>
        <button 
            onclick={resetTransform} 
            class="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-900 transition-colors"
            title="Recenter"
        >
            <Maximize2 size={16} />
        </button>
        <div class="h-4 w-px bg-slate-800 mx-1"></div>
        <button 
            onclick={loadTasks} 
            class="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-900 transition-colors"
            class:animate-spin={isLoading}
            title="Refresh Flows"
        >
            <RefreshCw size={16} />
        </button>
    </div>

    <!-- Main Panning viewport window -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div 
        class="w-full h-full cursor-grab active:cursor-grabbing overflow-hidden"
        onmousedown={handleMouseDown}
        onmousemove={handleMouseMove}
        onmouseup={handleMouseUp}
        onmouseleave={handleMouseUp}
        onwheel={handleWheel}
        ontouchstart={handleTouchStart}
        ontouchmove={handleTouchMove}
        ontouchend={handleTouchEnd}
        ontouchcancel={handleTouchEnd}
    >
        <!-- Scaled transform element container -->
        <div 
            class="relative origin-top-left w-full h-full"
            style="transform: translate({panX}px, {panY}px) scale({zoom});"
        >
            {#if isLoading && tasks.length === 0}
                <div class="absolute inset-0 flex flex-col items-center justify-center gap-3 opacity-60">
                    <RefreshCw class="w-8 h-8 text-sky-500 animate-spin" />
                    <p class="text-slate-400 text-xs italic font-medium">Spawning pipeline stages...</p>
                </div>
            {:else if tasks.length === 0}
                <div class="absolute inset-0 flex flex-col items-center justify-center gap-3 opacity-55">
                    <Brain class="w-12 h-12 text-slate-700 animate-bounce" />
                    <p class="text-slate-400 text-xs italic font-bold">No active background workflows</p>
                </div>
            {:else}
                <!-- SVG flowing connections -->
                <svg class="absolute inset-0 w-[5000px] h-[5000px] pointer-events-none overflow-visible">
                    <defs>
                        <filter id="glow-sky" x="-20%" y="-20%" width="140%" height="140%">
                            <feGaussianBlur stdDeviation="4" result="blur" />
                            <feMerge>
                                <feMergeNode in="blur" />
                                <feMergeNode in="SourceGraphic" />
                            </feMerge>
                        </filter>
                    </defs>

                    <!-- Connection lines between central agent core and active tasks -->
                    {#each visibleFilteredNodes as node}
                        {@const theme = getNodeTheme(node.status)}
                        <path 
                            d={calculateBezierPath(orchestratorNode, node)}
                            fill="none"
                            stroke={theme.accent}
                            stroke-width={3}
                            stroke-opacity={0.65}
                            filter="url(#glow-sky)"
                            class="transition-all duration-300"
                            class:pulse-flow={node.status === 'running'}
                        />
                    {/each}
                </svg>

                <!-- Central Virtual Agent Node card -->
                <div 
                    class="absolute w-[250px] min-h-[70px] rounded-2xl bg-slate-950/80 border border-sky-500/30 text-left p-4 flex flex-col justify-between group node-card shadow-[0_0_20px_rgba(59,130,246,0.1)] transition-all duration-300"
                    style="left: {orchestratorNode.x}px; top: {orchestratorNode.y}px;"
                >
                    <div class="flex items-center gap-3">
                        <div class="p-2 rounded-xl bg-sky-500/10 text-sky-400 animate-pulse">
                            <Sparkles size={18} />
                        </div>
                        <div class="min-w-0 flex-1">
                            <span class="text-[8px] font-black tracking-widest text-slate-500 uppercase font-mono block">
                                ORCHESTRATOR
                            </span>
                            <h4 class="text-xs font-bold text-slate-200 mt-0.5 truncate uppercase tracking-wide">
                                {orchestratorNode.label}
                            </h4>
                        </div>
                    </div>
                </div>

                <!-- HTML Task Nodes Render Loop -->
                {#each visibleFilteredNodes as node}
                    {@const theme = getNodeTheme(node.status)}
                    {@const Icon = theme.icon}
                    
                    <button 
                        onclick={() => handleNodeClick(node)}
                        class="absolute w-[250px] min-h-[90px] rounded-2xl bg-slate-950/75 border text-left p-4 hover:-translate-y-1 transition-all duration-250 cursor-pointer flex flex-col justify-between group node-card {theme.border} {theme.glow}"
                        style="left: {node.x}px; top: {node.y}px;"
                    >
                        <!-- Port circle anchors -->
                        <div 
                            class="absolute w-3 h-3 rounded-full bg-slate-950 border flex items-center justify-center z-10 group-hover:scale-110 transition-transform top-0 left-1/2 -translate-x-1/2 -translate-y-1.5"
                            style="border-color: {theme.accent};"
                        >
                            <div class="w-1.5 h-1.5 rounded-full" style="background-color: {theme.accent};"></div>
                        </div>

                        <!-- Card Header -->
                        <div class="flex items-start gap-3 w-full">
                            <div class="p-2 rounded-xl {theme.bg} {theme.text} group-hover:scale-110 transition-transform">
                                <Icon size={18} />
                            </div>
                            <div class="min-w-0 flex-1">
                                <span class="text-[8px] font-black tracking-widest text-slate-500 uppercase font-mono block">
                                    {node.status.replace(/_/g, ' ')}
                                </span>
                                <h4 class="text-xs font-bold text-slate-200 mt-0.5 truncate uppercase tracking-wide group-hover:text-white transition-colors">
                                    {node.title}
                                </h4>
                            </div>
                        </div>

                        <!-- Card Footer -->
                        <div class="flex items-center justify-between w-full mt-3 pt-2.5 border-t border-slate-900/60 text-[9px] text-slate-500 font-medium">
                            <span class="truncate pr-2 font-semibold text-slate-400">
                                {node.global_goal}
                            </span>
                            <span class="shrink-0 uppercase font-bold tracking-widest font-mono text-[8px] {theme.text}">
                                {formatTime(node.created_at)}
                            </span>
                        </div>
                    </button>
                {/each}
            {/if}
        </div>
    </div>
</div>

<style>
    /* Marching ants connection speed flow animation */
    .pulse-flow {
        stroke-dasharray: 6, 6;
        animation: march 25s linear infinite;
    }

    @keyframes march {
        to {
            stroke-dashoffset: -500;
        }
    }

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
