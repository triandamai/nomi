<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '$lib/api/client';
    import { popupStore } from '$lib/stores/popup.svelte';
    import { 
        ZoomIn, 
        ZoomOut, 
        Maximize2, 
        Search, 
        User, 
        MessageSquare, 
        Bell, 
        Calendar, 
        Brain, 
        DollarSign, 
        Heart, 
        Activity, 
        ArrowLeft, 
        RefreshCw,
        Settings,
        Lock,
        Share2,
        Cpu,
        Server
    } from 'lucide-svelte';
    import AutonomousTasksPopUp from '$lib/components/AutonomousTasksPopUp.svelte';
    import ProfileSettingsPopUp from '$lib/components/ProfileSettingsPopUp.svelte';
    import HealthHistoryPopUp from '$lib/components/HealthHistoryPopUp.svelte';

    interface WorkspaceNode {
        id: string;
        label: string;
        node_type: 'USER' | 'SYSTEM' | 'LOAD_MORE' | 'CONVERSATION' | 'REMINDER' | 'SCHEDULED_TASK' | 'AUTONOMOUS_TASK' | 'MONEY' | 'HEALTH' | 'CHANNEL' | 'SRP_PROPOSAL';
        status: string;
        subtitle?: string;
        info?: string;
        // Canvas positions
        x?: number;
        y?: number;
    }

    interface WorkspaceEdge {
        source: string;
        target: string;
        relation: string;
    }

    interface GraphData {
        nodes: WorkspaceNode[];
        edges: WorkspaceEdge[];
    }

    // State Variables
    let windowWidth = $state(1024);
    let isMobile = $derived(windowWidth < 768);
    let nodes = $state<WorkspaceNode[]>([]);
    let edges = $state<WorkspaceEdge[]>([]);
    let isLoading = $state(true);
    let isFetchingCategories = $state(false);
    let isFetchingItems = $state(false);
    let searchQuery = $state('');
    let usersOffset = $state(0);
    let isFetchingMoreUsers = $state(false);

    // Zoom & Pan Canvas Transform State
    let zoom = $state(0.9);
    let panX = $state(50);
    let panY = $state(50);
    let isDragging = $state(false);
    let startX = $state(0);
    let startY = $state(0);

    // Active Highlight/Selection
    let selectedNodeId = $state<string | null>(null);
    let selectedUserId = $state<string | null>(null);
    let selectedCategoryType = $state<string | null>(null);

    // 1. Position USER and SYSTEM nodes (always visible, Level 1)
    let positionedUserNodes = $derived.by(() => {
        const rootNodes = nodes.filter(n => n.node_type === 'USER' || n.node_type === 'SYSTEM');
        let userY = 150;
        const mapped = rootNodes.map(node => {
            const copy = { ...node };
            copy.x = 80;
            copy.y = userY;
            userY += 130;
            return copy;
        });

        // Add a "Load More" node at the bottom of the list if there are at least 10 users in the workspace
        const userNodesCount = nodes.filter(n => n.node_type === 'USER').length;
        if (userNodesCount >= 10 && userNodesCount % 10 === 0) {
            mapped.push({
                id: 'load-more-users-node',
                label: 'Load More Users',
                node_type: 'LOAD_MORE' as any,
                status: 'active',
                subtitle: 'Expand Directory',
                info: 'Click to load additional workspace profiles',
                x: 80,
                y: userY
            });
        }

        return mapped;
    });

    // 2. Position Category nodes (revealed on user click, Level 2)
    let visibleCategoryNodes = $derived.by(() => {
        if (!selectedUserId) return [];
        
        const catNodes = nodes.filter(n => n.node_type.startsWith('CATEGORY_') && n.id.includes(selectedUserId!));
        const rowHeight = 110;
        
        const selectedUser = positionedUserNodes.find(u => u.id === selectedUserId);
        const centerY = selectedUser ? selectedUser.y! : 200;
        
        let startY = centerY - ((catNodes.length - 1) * rowHeight) / 2;
        startY = Math.max(startY, 80);
        
        return catNodes.map((node, index) => {
            const categoryType = node.node_type.replace('CATEGORY_', '');
            return {
                ...node,
                categoryType,
                x: 420,
                y: startY + index * rowHeight
            };
        });
    });

    // 3. Position Item nodes (revealed on category click, Level 3)
    let visibleItemNodes = $derived.by(() => {
        if (!selectedUserId || !selectedCategoryType) return [];
        
        const catNodeId = `category-${selectedUserId}-${selectedCategoryType}`;
        const itemIds = edges
            .filter(e => e.source === catNodeId)
            .map(e => e.target);
            
        const items = nodes.filter(n => itemIds.includes(n.id));
        const rowHeight = 110;
        
        const selectedCatNode = visibleCategoryNodes.find(c => c.categoryType === selectedCategoryType);
        const catCenterY = selectedCatNode ? selectedCatNode.y! : 200;
        
        let itemStartY = catCenterY - ((items.length - 1) * rowHeight) / 2;
        itemStartY = Math.max(itemStartY, 80);
        
        return items.map((item, index) => {
            const copy = { ...item };
            copy.x = 760;
            copy.y = itemStartY + index * rowHeight;
            return copy;
        });
    });

    // Combine all currently visible nodes
    let visibleNodes = $derived.by(() => {
        return [...positionedUserNodes, ...visibleCategoryNodes, ...visibleItemNodes];
    });

    // Combine all currently visible edges
    let visibleEdges = $derived.by(() => {
        const edgesList: WorkspaceEdge[] = [];
        if (!selectedUserId) return edgesList;

        // Add category contains edges
        edges.forEach(e => {
            if (e.source === selectedUserId) {
                edgesList.push(e);
            }
        });

        // Add selected category items edges
        if (selectedCategoryType) {
            const catNodeId = `category-${selectedUserId}-${selectedCategoryType}`;
            edges.forEach(e => {
                if (e.source === catNodeId) {
                    edgesList.push(e);
                }
            });
        }

        return edgesList;
    });

    // Filter visible nodes by search query
    let visibleFilteredNodes = $derived(
        visibleNodes.map(node => ({
            ...node,
            isHighlighted: searchQuery ? (
                node.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
                node.node_type.toLowerCase().includes(searchQuery.toLowerCase()) ||
                (node.subtitle && node.subtitle.toLowerCase().includes(searchQuery.toLowerCase()))
            ) : true
        }))
    );

    // API Data retrieval (Initial Load Level 1: Fetch Users)
    async function loadWorkspaceGraph() {
        isLoading = true;
        try {
            const res = await api.get<GraphData>('/graph/workspace');
            if (res.data) {
                nodes = res.data.nodes;
                edges = res.data.edges;
            }
        } catch (e) {
            console.error('Failed to load workspace graph users:', e);
        } finally {
            isLoading = false;
        }
    }

    // Paginated load more users
    async function loadMoreUsers() {
        if (isFetchingMoreUsers) return;
        isFetchingMoreUsers = true;
        usersOffset += 10;
        try {
            const res = await api.get<GraphData>(`/graph/workspace?offset=${usersOffset}`);
            if (res.data) {
                const newNodes = res.data.nodes.filter(n => !nodes.some(existing => existing.id === n.id));
                const newEdges = res.data.edges.filter(e => !edges.some(existing => existing.source === e.source && existing.target === e.target));
                nodes = [...nodes, ...newNodes];
                edges = [...edges, ...newEdges];
            }
        } catch (e) {
            console.error('Failed to load more users:', e);
        } finally {
            isFetchingMoreUsers = false;
        }
    }

    // Fetch Categories for a given user (Level 2: Fetch Categories)
    async function loadCategories(userId: string) {
        isFetchingCategories = true;
        try {
            const res = await api.get<GraphData>(`/graph/workspace?user_id=${encodeURIComponent(userId)}`);
            if (res.data) {
                // Merge category nodes and contains edges
                const newNodes = res.data.nodes.filter(n => !nodes.some(existing => existing.id === n.id));
                const newEdges = res.data.edges.filter(e => !edges.some(existing => existing.source === e.source && existing.target === e.target));
                
                nodes = [...nodes, ...newNodes];
                edges = [...edges, ...newEdges];
            }
        } catch (e) {
            console.error('Failed to load workspace graph categories:', e);
        } finally {
            isFetchingCategories = false;
        }
    }

    // Fetch Items for a given category (Level 3: Fetch Items)
    async function loadCategoryItems(userId: string, category: string) {
        isFetchingItems = true;
        try {
            const res = await api.get<GraphData>(`/graph/workspace?user_id=${encodeURIComponent(userId)}&category=${encodeURIComponent(category)}`);
            if (res.data) {
                // Merge item nodes and contains edges
                const newNodes = res.data.nodes.filter(n => !nodes.some(existing => existing.id === n.id));
                const newEdges = res.data.edges.filter(e => !edges.some(existing => existing.source === e.source && existing.target === e.target));
                
                nodes = [...nodes, ...newNodes];
                edges = [...edges, ...newEdges];
            }
        } catch (e) {
            console.error('Failed to load workspace graph category items:', e);
        } finally {
            isFetchingItems = false;
        }
    }

    // Canvas Zoom & Pan Mouse Listeners
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

    // Canvas Touch Drag & Pinch-to-Zoom Listeners
    let isTouching = $state(false);
    let initialTouchDist = $state(0);
    let startZoom = $state(1.0);

    function handleTouchStart(e: TouchEvent) {
        if ((e.target as HTMLElement).closest('.node-card')) return;
        isTouching = true;
        
        if (e.touches.length === 1) {
            // Single finger drag
            startX = e.touches[0].clientX - panX;
            startY = e.touches[0].clientY - panY;
        } else if (e.touches.length === 2) {
            // Double finger pinch-to-zoom
            initialTouchDist = Math.hypot(
                e.touches[0].clientX - e.touches[1].clientX,
                e.touches[0].clientY - e.touches[1].clientY
            );
            startZoom = zoom;
            
            // Re-center touch drag anchor in the middle of both fingers
            startX = (e.touches[0].clientX + e.touches[1].clientX) / 2 - panX;
            startY = (e.touches[0].clientY + e.touches[1].clientY) / 2 - panY;
        }
    }

    function handleTouchMove(e: TouchEvent) {
        if (!isTouching) return;

        if (e.touches.length === 1) {
            // Move/drag
            panX = e.touches[0].clientX - startX;
            panY = e.touches[0].clientY - startY;
        } else if (e.touches.length === 2 && initialTouchDist > 0) {
            // Pinch to zoom
            const dist = Math.hypot(
                e.touches[0].clientX - e.touches[1].clientX,
                e.touches[0].clientY - e.touches[1].clientY
            );
            zoom = Math.max(0.4, Math.min(startZoom * (dist / initialTouchDist), 2.0));

            // Adjust panning in parallel so it zooms centered around touch midpoint
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
            panX = 10;
            panY = 30;
        } else {
            zoom = 0.85;
            panX = 80;
            panY = 60;
        }
    }

    // Node Type Helpers: Colors, Borders & Neon Glows
    function getNodeTheme(type: string) {
        if (type.startsWith('CATEGORY_')) {
            const baseType = type.replace('CATEGORY_', '');
            const baseTheme = getNodeTheme(baseType);
            return {
                ...baseTheme,
                border: 'border-dashed border-2 ' + baseTheme.border.split(' ')[0].replace('/40', ''),
                glow: 'opacity-90 shadow-[0_0_15px_rgba(255,255,255,0.05)]'
            };
        }

        switch (type) {
            case 'LOAD_MORE': 
                return {
                    border: 'border-slate-700/60 hover:border-slate-500 hover:scale-102 border-dashed shadow-[0_0_10px_rgba(255,255,255,0.02)]',
                    text: 'text-slate-400 group-hover:text-slate-200',
                    bg: 'bg-slate-950/20 hover:bg-slate-900/40',
                    glow: '',
                    accent: '#475569',
                    icon: RefreshCw
                };
            case 'SYSTEM': 
                return {
                    border: 'border-fuchsia-500/40 hover:border-fuchsia-500 shadow-fuchsia-500/10',
                    text: 'text-fuchsia-400',
                    bg: 'bg-fuchsia-500/5',
                    glow: 'shadow-[0_0_20px_rgba(217,70,239,0.15)]',
                    accent: '#d946ef',
                    icon: Server
                };
            case 'USER': 
                return {
                    border: 'border-blue-500/40 hover:border-blue-500 shadow-blue-500/10',
                    text: 'text-blue-400',
                    bg: 'bg-blue-500/5',
                    glow: 'shadow-[0_0_20px_rgba(59,130,246,0.15)]',
                    accent: '#3b82f6',
                    icon: User
                };
            case 'CONVERSATION': 
                return {
                    border: 'border-violet-500/40 hover:border-violet-500 shadow-violet-500/10',
                    text: 'text-violet-400',
                    bg: 'bg-violet-500/5',
                    glow: 'shadow-[0_0_20px_rgba(139,92,246,0.15)]',
                    accent: '#8b5cf6',
                    icon: MessageSquare
                };
            case 'REMINDER': 
                return {
                    border: 'border-emerald-500/40 hover:border-emerald-500 shadow-emerald-500/10',
                    text: 'text-emerald-400',
                    bg: 'bg-emerald-500/5',
                    glow: 'shadow-[0_0_20px_rgba(16,185,129,0.15)]',
                    accent: '#10b981',
                    icon: Bell
                };
            case 'SCHEDULED_TASK': 
                return {
                    border: 'border-amber-500/40 hover:border-amber-500 shadow-amber-500/10',
                    text: 'text-amber-400',
                    bg: 'bg-amber-500/5',
                    glow: 'shadow-[0_0_20px_rgba(245,158,11,0.15)]',
                    accent: '#f59e0b',
                    icon: Calendar
                };
            case 'AUTONOMOUS_TASK': 
                return {
                    border: 'border-pink-500/40 hover:border-pink-500 shadow-pink-500/10',
                    text: 'text-pink-400',
                    bg: 'bg-pink-500/5',
                    glow: 'shadow-[0_0_20px_rgba(236,72,153,0.15)]',
                    accent: '#ec4899',
                    icon: Brain
                };
            case 'MONEY': 
                return {
                    border: 'border-cyan-500/40 hover:border-cyan-500 shadow-cyan-500/10',
                    text: 'text-cyan-400',
                    bg: 'bg-cyan-500/5',
                    glow: 'shadow-[0_0_20px_rgba(6,182,212,0.15)]',
                    accent: '#06b6d4',
                    icon: DollarSign
                };
            case 'HEALTH': 
                return {
                    border: 'border-rose-500/40 hover:border-rose-500 shadow-rose-500/10',
                    text: 'text-rose-400',
                    bg: 'bg-rose-500/5',
                    glow: 'shadow-[0_0_20px_rgba(244,63,94,0.15)]',
                    accent: '#f43f5e',
                    icon: Heart
                };
            case 'CHANNEL': 
                return {
                    border: 'border-orange-500/40 hover:border-orange-500 shadow-orange-500/10',
                    text: 'text-orange-400',
                    bg: 'bg-orange-500/5',
                    glow: 'shadow-[0_0_20px_rgba(249,115,22,0.15)]',
                    accent: '#f97316',
                    icon: Share2
                };
            case 'SRP_PROPOSAL': 
                return {
                    border: 'border-indigo-500/40 hover:border-indigo-500 shadow-indigo-500/10',
                    text: 'text-indigo-400',
                    bg: 'bg-indigo-500/5',
                    glow: 'shadow-[0_0_20px_rgba(99,102,241,0.15)]',
                    accent: '#6366f1',
                    icon: Cpu
                };
            default: 
                return {
                    border: 'border-slate-800 hover:border-slate-600',
                    text: 'text-slate-400',
                    bg: 'bg-slate-500/5',
                    glow: '',
                    accent: '#64748b',
                    icon: Activity
                };
        }
    }

    // Cubic Bezier Connector Line Calculation (Input -> Output)
    function calculateBezierPath(source: WorkspaceNode, target: WorkspaceNode) {
        if (!source || !target || source.x === undefined || source.y === undefined || target.x === undefined || target.y === undefined) return '';
        
        if (isMobile) {
            // Output Port at bottom of card
            const x1 = source.x + 125;
            const y1 = source.y + 90;
            // Input Port at top of card
            const x2 = target.x + 125;
            const y2 = target.y;

            // Signature vertical wave path
            const controlOffset = 60;
            return `M ${x1} ${y1} C ${x1} ${y1 + controlOffset}, ${x2} ${y2 - controlOffset}, ${x2} ${y2}`;
        } else {
            // Check if source and target are horizontally aligned (vertical flow directly down)
            const isVertical = Math.abs(source.x - target.x) < 50;

            if (isVertical) {
                // Output Port at bottom center of card
                const x1 = source.x + 125;
                const y1 = source.y + 90;
                // Input Port at top center of card
                const x2 = target.x + 125;
                const y2 = target.y;

                const controlOffset = 40;
                return `M ${x1} ${y1} C ${x1} ${y1 + controlOffset}, ${x2} ${y2 - controlOffset}, ${x2} ${y2}`;
            } else {
                // Output Port on the right side of the card
                const x1 = source.x + 250;
                const y1 = source.y + 45;
                // Input Port on the left side of the card
                const x2 = target.x;
                const y2 = target.y + 45;

                // Signature horizontal wave path
                const controlOffset = 90;
                return `M ${x1} ${y1} C ${x1 + controlOffset} ${y1}, ${x2 - controlOffset} ${y2}, ${x2} ${y2}`;
            }
        }
    }

    function handleNodeClick(node: any) {
        if (node.node_type === 'LOAD_MORE') {
            loadMoreUsers();
        } else if (node.node_type === 'USER' || node.node_type === 'SYSTEM') {
            // Clicked a user or system node: expand categories
            selectedUserId = node.id;
            selectedCategoryType = null; // reset category
            loadCategories(node.id);
        } else if (node.node_type.startsWith('CATEGORY_')) {
            // Clicked a category node: expand items
            selectedCategoryType = node.categoryType;
            if (selectedUserId) {
                loadCategoryItems(selectedUserId, node.categoryType);
            }
        } else {
            // Clicked an item node: open its popup exactly as before!
            selectedNodeId = node.id;

            switch (node.node_type) {
                case 'HEALTH':
                    popupStore.open({
                        title: 'Health & Vitality Metrics',
                        width: 'max-w-2xl',
                        contentSnippet: healthMetricsSnippet
                    });
                    break;
                case 'AUTONOMOUS_TASK':
                    popupStore.open({
                        title: 'Autonomous Agent Tasks',
                        width: 'max-w-3xl h-[100dvh]',
                        contentSnippet: autonomousTasksSnippet
                    });
                    break;
                case 'CONVERSATION':
                    popupStore.open({
                        title: `Conversation: ${node.label}`,
                        width: 'max-w-2xl',
                        contentSnippet: conversationDetailSnippet
                    });
                    break;
                case 'SRP_PROPOSAL':
                    popupStore.open({
                        title: `SRP Proposal: ${node.label}`,
                        width: 'max-w-2xl',
                        contentSnippet: srpProposalSnippet
                    });
                    break;
                default:
                    popupStore.open({
                        title: node.label,
                        width: 'max-w-lg',
                        contentSnippet: defaultNodeSnippet
                    });
                    break;
            }
        }
    }

    onMount(() => {
        windowWidth = window.innerWidth;
        loadWorkspaceGraph();
        resetTransform();
    });
</script>

<!-- Dynamic Svelte Snippets for popup render -->
{#snippet profileSettingsSnippet()}
    <ProfileSettingsPopUp />
{/snippet}

{#snippet healthMetricsSnippet()}
    <HealthHistoryPopUp />
{/snippet}

{#snippet autonomousTasksSnippet()}
    <div class="h-full">
        <AutonomousTasksPopUp />
    </div>
{/snippet}

{#snippet conversationDetailSnippet()}
    <div class="p-6 bg-[#0b0f19] text-slate-100 rounded-2xl border border-slate-800 space-y-6">
        <div class="space-y-2">
            <span class="text-[9px] font-black uppercase tracking-widest text-slate-500 font-mono">Channel Node</span>
            <h3 class="text-lg font-black text-white uppercase tracking-wide">
                {nodes.find(n => n.id === selectedNodeId)?.label}
            </h3>
            <p class="text-xs text-slate-400">
                Configuration dashboard and Dynamic Execution Boundaries (DEB) for this communication socket.
            </p>
        </div>

        <div class="grid grid-cols-2 gap-4 pt-4 border-t border-slate-900">
            <div class="p-4 bg-slate-950/60 border border-slate-800 rounded-xl space-y-1">
                <span class="text-[9px] font-bold text-slate-500 uppercase font-mono">Boundary Sociability</span>
                <p class="text-xs font-bold text-sky-400">Proactive (Always Participate)</p>
            </div>
            <div class="p-4 bg-slate-950/60 border border-slate-800 rounded-xl space-y-1">
                <span class="text-[9px] font-bold text-slate-500 uppercase font-mono">Bound Guardrails</span>
                <p class="text-xs font-bold text-emerald-400">Strict (Prompts Hardened)</p>
            </div>
        </div>

        <div class="flex items-center gap-2 p-3 bg-amber-500/5 border border-amber-500/20 text-amber-400 text-xs rounded-xl">
            <Lock size={16} />
            <span>To adjust live DEB sociability triggers, open settings inside the primary Chat viewport.</span>
        </div>
    </div>
{/snippet}

{#snippet srpProposalSnippet()}
    {@const selectedNode = nodes.find(n => n.id === selectedNodeId)}
    {#if selectedNode}
        <div class="p-6 bg-[#0b0f19] text-slate-100 rounded-2xl border border-slate-800 space-y-6">
            <div class="space-y-2">
                <span class="text-[9px] font-black uppercase tracking-widest text-indigo-400 font-mono">Self-Refining Plugin Proposal</span>
                <h3 class="text-xl font-black text-white uppercase tracking-wide">
                    {selectedNode.label}
                </h3>
                <p class="text-xs text-indigo-300 font-mono">
                    {selectedNode.subtitle}
                </p>
            </div>


            <div class="flex items-center justify-between p-4 bg-slate-950/60 border border-slate-900 rounded-xl">
                <span class="text-[9px] font-bold text-slate-500 uppercase font-mono">Build Queue Status</span>
                <div class="flex items-center gap-2">
                    <span class="w-2.5 h-2.5 rounded-full animate-pulse
                        {selectedNode.status === 'deployed' ? 'bg-emerald-500 shadow-[0_0_10px_#10b981]' : 
                         selectedNode.status === 'ready' ? 'bg-blue-500 shadow-[0_0_10px_#3b82f6]' : 
                         selectedNode.status === 'processing' ? 'bg-amber-500 shadow-[0_0_10px_#f59e0b]' : 
                         'bg-slate-500'}"
                    ></span>
                    <span class="text-xs font-bold uppercase tracking-wider font-mono
                        {selectedNode.status === 'deployed' ? 'text-emerald-400' : 
                         selectedNode.status === 'ready' ? 'text-blue-400' : 
                         selectedNode.status === 'processing' ? 'text-amber-400' : 
                         'text-slate-400'}"
                    >
                        {selectedNode.status || 'unknown'}
                    </span>
                </div>
            </div>

            <div class="flex items-center gap-2 p-3 bg-indigo-500/5 border border-indigo-500/20 text-indigo-400 text-xs rounded-xl">
                <Cpu size={16} class="animate-spin" style="animation-duration: 4s" />
                <span>SWE-Agent is refining code generation. Auto-deployment initiates upon status: Ready.</span>
            </div>
        </div>
    {/if}
{/snippet}

{#snippet defaultNodeSnippet()}
    {@const selectedNode = nodes.find(n => n.id === selectedNodeId)}
    {#if selectedNode}
        <div class="p-6 bg-[#0b0f19] text-slate-100 rounded-2xl border border-slate-800 space-y-4">
            <div class="flex items-start justify-between gap-4">
                <div class="space-y-1">
                    <span class="text-[9px] font-black uppercase tracking-widest text-slate-500 font-mono">{selectedNode.node_type} RECORD</span>
                    <h3 class="text-base font-extrabold text-white">{selectedNode.label}</h3>
                </div>
                <span class="px-2 py-0.5 rounded text-[9px] font-black uppercase tracking-widest border font-mono bg-slate-950 border-slate-800 text-sky-400">
                    {selectedNode.status}
                </span>
            </div>

            {#if selectedNode.subtitle || selectedNode.info}
                <div class="p-4 bg-slate-950/80 border border-slate-900 rounded-xl space-y-2">
                    {#if selectedNode.subtitle}
                        <p class="text-xs text-slate-300 font-semibold">{selectedNode.subtitle}</p>
                    {/if}
                    {#if selectedNode.info}
                        <p class="text-xs text-slate-400 leading-relaxed italic">{selectedNode.info}</p>
                    {/if}
                </div>
            {/if}
            
            <div class="flex items-center gap-2 text-[10px] text-slate-500 font-mono pt-3 border-t border-slate-900">
                <Settings size={12} />
                <span>ID: {selectedNode.id}</span>
            </div>
        </div>
    {/if}
{/snippet}

<svelte:window bind:innerWidth={windowWidth} onresize={resetTransform} />

<!-- Full-Screen Interactive Canvas Wrapper -->
<div class="relative w-full h-[100dvh] bg-[#030712] text-slate-100 overflow-hidden font-sans select-none">
    
    <!-- n8n Styled Dot Pattern Background -->
    <div 
        class="absolute inset-0 pointer-events-none transition-all duration-75"
        style="
            background-image: radial-gradient(#1e293b 1px, transparent 1px);
            background-size: {24 * zoom}px {24 * zoom}px;
            background-position: {panX}px {panY}px;
            opacity: 0.85;
        "
    ></div>

    <!-- Header Floating Deck -->
    <div class="absolute top-6 left-6 right-6 z-20 flex flex-col sm:flex-row gap-4 items-center justify-between pointer-events-none">
        
        <!-- Left Title Anchor -->
        <div class="flex items-center gap-3 bg-slate-950/85 backdrop-blur-md border border-slate-800/80 px-5 py-3 rounded-2xl shadow-xl pointer-events-auto w-full sm:w-auto justify-between sm:justify-start">
            <div class="flex items-center gap-3">
                <button onclick={() => window.history.back()} class="p-1.5 rounded-xl hover:bg-slate-900 text-slate-400 hover:text-white transition-colors">
                    <ArrowLeft size={16} />
                </button>
                <div class="h-4 w-px bg-slate-800"></div>
                <div class="flex items-center gap-2">
                    <Activity class="w-4 h-4 text-sky-500 animate-pulse" />
                    <span class="text-xs font-black uppercase tracking-[0.2em] text-slate-300">Nomi Canvas</span>
                </div>
            </div>
        </div>

        <!-- Right Controller: Search & Spotlight -->
        <div class="flex items-center gap-3 bg-slate-950/85 backdrop-blur-md border border-slate-800/80 px-4 py-2.5 rounded-2xl shadow-xl pointer-events-auto w-full sm:w-[280px]">
            <Search size={16} class="text-slate-500 shrink-0" />
            <input 
                type="text" 
                bind:value={searchQuery}
                placeholder="Search graph nodes..." 
                class="bg-transparent border-none text-xs text-slate-200 outline-none w-full placeholder:text-slate-700 font-medium"
            />
        </div>
    </div>

    <!-- Bottom Controls Floating Deck -->
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
            onclick={loadWorkspaceGraph} 
            class="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-900 transition-colors"
            class:animate-spin={isLoading}
            title="Refresh Graph"
        >
            <RefreshCw size={16} />
        </button>
    </div>

    <!-- Interactive Interactive Canvas Canvas Window -->
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
        <!-- Scaled Panning Transformer viewport -->
        <div 
            class="relative origin-top-left w-full h-full"
            style="transform: translate({panX}px, {panY}px) scale({zoom});"
        >
            {#if isLoading && nodes.length === 0}
                <div class="absolute inset-0 flex flex-col items-center justify-center gap-3 opacity-60">
                    <RefreshCw class="w-8 h-8 text-sky-500 animate-spin" />
                    <p class="text-slate-400 text-xs italic font-medium">Recompiling workspace nodes...</p>
                </div>
            {:else}
                <!-- SVG Connections Layer -->
                <svg class="absolute inset-0 w-[5000px] h-[5000px] pointer-events-none overflow-visible">
                    <defs>
                        <!-- Glow filters and active gradient paths -->
                        <filter id="glow-blue" x="-20%" y="-20%" width="140%" height="140%">
                            <feGaussianBlur stdDeviation="4" result="blur" />
                            <feMerge>
                                <feMergeNode in="blur" />
                                <feMergeNode in="SourceGraphic" />
                            </feMerge>
                        </filter>
                    </defs>

                    <!-- Render active lines linking parent-to-child relations -->
                    {#each visibleEdges as edge}
                        {@const sourceNode = visibleNodes.find(n => n.id === edge.source)}
                        {@const targetNode = visibleNodes.find(n => n.id === edge.target)}
                        {#if sourceNode && targetNode}
                            {@const theme = getNodeTheme(targetNode.node_type)}
                            {@const isHighlighted = (!searchQuery) || (
                                sourceNode.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
                                targetNode.label.toLowerCase().includes(searchQuery.toLowerCase())
                            )}
                            
                            <!-- Cubic Bezier Curve representing direct relationship -->
                            <path 
                                d={calculateBezierPath(sourceNode, targetNode)}
                                fill="none"
                                stroke={theme.accent}
                                stroke-width={3}
                                stroke-opacity={0.65}
                                filter="url(#glow-blue)"
                                class="transition-all duration-300"
                                class:pulse-flow={targetNode.status === 'running' || targetNode.status === 'active'}
                            />
                        {/if}
                    {/each}
                </svg>

                <!-- HTML Nodes Layer -->
                {#if isFetchingCategories}
                    {@const clickedUserNode = positionedUserNodes.find(u => u.id === selectedUserId)}
                    {@const loaderTop = clickedUserNode ? clickedUserNode.y : 200}
                    <div 
                        class="absolute w-[250px] min-h-[90px] rounded-2xl bg-slate-950/40 border border-slate-900 border-dashed p-4 flex items-center justify-center gap-3 animate-pulse pointer-events-none"
                        style="left: 420px; top: {loaderTop}px;"
                    >
                        <RefreshCw class="w-4 h-4 text-sky-500 animate-spin" />
                        <span class="text-[10px] font-mono text-slate-500 uppercase tracking-wider">Hydrating Folders...</span>
                    </div>
                {/if}

                {#if isFetchingItems}
                    {@const clickedCatNode = visibleCategoryNodes.find(c => c.categoryType === selectedCategoryType)}
                    {@const loaderTop = clickedCatNode ? clickedCatNode.y : 200}
                    <div 
                        class="absolute w-[250px] min-h-[90px] rounded-2xl bg-slate-950/40 border border-slate-900 border-dashed p-4 flex items-center justify-center gap-3 animate-pulse pointer-events-none"
                        style="left: 760px; top: {loaderTop}px;"
                    >
                        <RefreshCw class="w-4 h-4 text-emerald-500 animate-spin" />
                        <span class="text-[10px] font-mono text-slate-500 uppercase tracking-wider">Hydrating Leaves...</span>
                    </div>
                {/if}

                {#each visibleFilteredNodes as node}
                    {@const theme = getNodeTheme(node.node_type)}
                    {@const Icon = theme.icon}
                    
                    <!-- Dynamic absolutely positioned n8n card node -->
                    <button 
                        onclick={() => handleNodeClick(node)}
                        class="absolute w-[250px] min-h-[90px] rounded-2xl bg-slate-950/75 border text-left p-4 hover:-translate-y-1 transition-all duration-250 cursor-pointer flex flex-col justify-between group node-card {theme.border} {theme.glow}"
                        class:opacity-100={node.isHighlighted}
                        class:opacity-15={!node.isHighlighted}
                        class:ring-2={selectedUserId === node.id || selectedCategoryType === node.categoryType}
                        class:ring-sky-500={selectedUserId === node.id || selectedCategoryType === node.categoryType}
                        style="left: {node.x}px; top: {node.y}px;"
                    >
                        <!-- n8n Port Anchors (Circle Ports) -->
                        <!-- Input Port (Left side on desktop, Top side on mobile) -->
                        {#if node.node_type !== 'USER' && node.node_type !== 'SYSTEM' && node.node_type !== 'LOAD_MORE'}
                            <div 
                                class="absolute w-3 h-3 rounded-full bg-slate-950 border flex items-center justify-center z-10 group-hover:scale-110 transition-transform {isMobile ? 'top-0 left-1/2 -translate-x-1/2 -translate-y-1.5' : 'top-1/2 -left-1.5 -translate-y-1/2'}"
                                style="border-color: {theme.accent};"
                            >
                                <div class="w-1.5 h-1.5 rounded-full" style="background-color: {theme.accent};"></div>
                            </div>
                        {/if}

                        <!-- Output Port (Right side on desktop, Bottom side on mobile) -->
                        {#if node.node_type === 'USER' || node.node_type === 'SYSTEM' || node.node_type === 'CONVERSATION' || node.node_type.startsWith('CATEGORY_')}
                            <div 
                                class="absolute w-3 h-3 rounded-full bg-slate-950 border flex items-center justify-center z-10 group-hover:scale-110 transition-transform {isMobile ? 'bottom-0 left-1/2 -translate-x-1/2 translate-y-1.5' : 'top-1/2 -right-1.5 -translate-y-1/2'}"
                                style="border-color: {theme.accent};"
                            >
                                <div class="w-1.5 h-1.5 rounded-full" style="background-color: {theme.accent};"></div>
                            </div>
                        {/if}

                        <!-- Content inside n8n Node Card -->
                        <div class="flex items-start gap-3 w-full">
                            <div class="p-2 rounded-xl {theme.bg} {theme.text} group-hover:scale-110 transition-transform">
                                <Icon size={18} />
                            </div>
                            <div class="min-w-0 flex-1">
                                <span class="text-[8px] font-black tracking-widest text-slate-500 uppercase font-mono block">
                                    {node.node_type.replace(/_/g, ' ')}
                                </span>
                                <h4 class="text-xs font-bold text-slate-200 mt-0.5 truncate uppercase tracking-wide group-hover:text-white transition-colors">
                                    {node.label}
                                </h4>
                            </div>
                        </div>

                        <!-- Footer Node Status / Subtitles -->
                        <div class="flex items-center justify-between w-full mt-3 pt-2.5 border-t border-slate-900/60 text-[9px] text-slate-500 font-medium">
                            <span class="truncate pr-2 font-semibold text-slate-400">
                                {node.subtitle || node.node_type.toLowerCase()}
                            </span>
                            <span class="shrink-0 uppercase font-bold tracking-widest font-mono text-[8px] {theme.text}">
                                {(node.node_type === 'SRP_PROPOSAL' || node.node_type === 'AUTONOMOUS_TASK') ? node.status : (node.info || node.status)}
                            </span>
                        </div>
                    </button>
                {/each}
            {/if}
        </div>
    </div>
</div>

<style>
    /* Marching Ants neon animation for data active flow */
    .pulse-flow {
        stroke-dasharray: 6, 6;
        animation: march 25s linear infinite;
    }

    @keyframes march {
        to {
            stroke-dashoffset: -500;
        }
    }
</style>
