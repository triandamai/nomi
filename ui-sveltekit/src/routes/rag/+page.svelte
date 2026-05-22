<script lang="ts">
    import {onMount} from 'svelte';
    import {AlertCircle, Camera, Info, Loader2, Network, RefreshCw, Search, X} from 'lucide-svelte';
    import {type Node, ragStore} from '$lib/stores/rag.svelte';
    import {conversationStore} from '$lib/stores/conversation.svelte';
    import {popupStore} from '$lib/stores/popup.svelte';
    import * as THREE from 'three';
    import {eventBus, mdIt, setupMarkdownHelpers} from "$lib/utils";
    import {browser} from "$app/environment";

    let graphContainer: HTMLElement;
    let selectedNode = $state<Node | null>(null);
    let graphInstance: any = null;

    let searchQuery = $state('');
    let showDropdown = $state(false);
    let highlightedNodeId = $state<string | null>(null);

    let animationFrameId: number;
    let isInteracting = false;
    let orbitAngle = 0;
    const ORBIT_DISTANCE = 400;
    let interactionTimeout: ReturnType<typeof setTimeout>;

    // Watch activeConversationId and re-fetch graph data
    $effect(() => {
        const convId = conversationStore.activeConversationId;
        console.log("Active conversation changed, fetching graph for:", convId);
        ragStore.fetchGraph(convId);
        if (graphInstance) {
            resetCamera();
        }
    });

    onMount(() => {
        setupMarkdownHelpers();
        let isMounted = true;
        let cleanup = () => {
        };
        eventBus.emit("load",{})
        if(browser && graphContainer) {
            Promise.all([
                import('3d-force-graph'),
                import('three-spritetext')
            ]).then(([module, spriteTextModule]) => {
                if (!isMounted) return;

                const ForceGraph3D = module.default || module;
                const SpriteText = spriteTextModule.default || spriteTextModule;
                ragStore.fetchGraph(conversationStore.activeConversationId);

                if (graphContainer) {
                    setTimeout(() => {
                        try {
                            graphInstance = (ForceGraph3D as any)()(graphContainer)
                                .backgroundColor('#020617') // slate-950 deep black
                                .nodeId('id')
                                .nodeLabel((node: any) => `${node.label || 'Unknown'} (${node.node_type || 'Entity'})${node.conversation_id && node.conversation_id !== 'global' ? ' [Current Soul]' : ' [Global]'}`)
                                .nodeAutoColorBy('node_type')
                                .nodeThreeObject((node: any) => {
                                    const nodeType = String(node.node_type || '').toLowerCase();
                                    const isSummary = nodeType === 'summary' || node.id === 'summary';
                                    const isLocal = node.conversation_id && node.conversation_id !== 'global';

                                    // Defensive size check
                                    let size = isSummary ? 12 : 5;
                                    if (isNaN(size) || size <= 0) size = 5;

                                    let color = node.id === highlightedNodeId ? '#ffffff' : (node.color || '#94a3b8');

                                    // Brighter color for local nodes
                                    if (isLocal && node.id !== highlightedNodeId) {
                                        // Make the color brighter/more saturated
                                        const c = new THREE.Color(color);
                                        c.offsetHSL(0, 0.2, 0.1);
                                        color = `#${c.getHexString()}`;
                                    }

                                    const material = new THREE.MeshPhongMaterial({
                                        color: color,
                                        transparent: true,
                                        opacity: 0.9,
                                        shininess: isLocal ? 100 : 30,
                                        emissive: isLocal ? color : 0x000000,
                                        emissiveIntensity: isLocal ? 0.5 : 0
                                    });

                                    const geometry = new THREE.SphereGeometry(size, 32, 32);
                                    const mesh = new THREE.Mesh(geometry, material);

                                    node.__sphereMesh = mesh;
                                    node.__baseSize = size;

                                    // Create a group to hold both the sphere and the permanent label
                                    const group = new THREE.Group();
                                    group.add(mesh);

                                    // Add permanent label using SpriteText
                                    const sprite = new (SpriteText as any)(node.label || 'Unknown');
                                    sprite.color = node.id === highlightedNodeId ? '#ffffff' : (isLocal ? '#f8fafc' : '#94a3b8'); // slate-50 or slate-400
                                    sprite.textHeight = isSummary ? 8 : 4;
                                    sprite.position.y = size + (isSummary ? 10 : 6);
                                    group.add(sprite);

                                    node.__threeObj = group;
                                    node.__labelSprite = sprite;

                                    return group;
                                })
                                .linkDirectionalParticles(2)
                                .linkDirectionalParticleWidth(1.5)
                                .linkDirectionalParticleColor((link: any) => {
                                    const sourceNode = typeof link.source === 'object' ? link.source : ragStore.graphData.nodes.find((n: any) => n.id === link.source);
                                    return sourceNode?.color || '#94a3b8';
                                })
                                .linkDirectionalParticleSpeed(0.005)
                                .linkCurvature(0.2)
                                .linkColor((link: any) => {
                                    const sourceNode = typeof link.source === 'object' ? link.source : ragStore.graphData.nodes.find((n: any) => n.id === link.source);
                                    return sourceNode?.color ? `${sourceNode.color}40` : '#33415540';
                                })
                                .linkOpacity(0.3)
                                .onNodeClick((node: any) => {
                                    openNodeInfo(node);
                                    handleInteraction();

                                    const distance = 120;
                                    const x = node.x || 0;
                                    const y = node.y || 0;
                                    const z = node.z || 0;
                                    const currentDist = Math.hypot(x, y, z);
                                    const distRatio = currentDist === 0 ? 2 : 1 + distance / currentDist;

                                    graphInstance.cameraPosition(
                                        {x: x * distRatio, y: y * distRatio, z: z * distRatio},
                                        node,
                                        2000
                                    );
                                })
                                .width(graphContainer.clientWidth)
                                .height(graphContainer.clientHeight);

                            if (ragStore.graphData && ragStore.graphData.nodes.length > 0) {
                                graphInstance.graphData(ragStore.graphData);
                            }

                            const scene = graphInstance.scene();
                            scene.add(new THREE.AmbientLight(0xffffff, 0.6));
                            const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
                            dirLight.position.set(1, 2, 3);
                            scene.add(dirLight);

                            window.addEventListener('resize', handleResize);
                            graphContainer.addEventListener('mousedown', handleInteraction);
                            graphContainer.addEventListener('wheel', handleInteraction);

                            let time = 0;
                            const animate = () => {
                                time += 0.05;
                                if (!isInteracting && graphInstance) {
                                    orbitAngle += Math.PI / 3000;
                                    graphInstance.cameraPosition({
                                        x: ORBIT_DISTANCE * Math.sin(orbitAngle),
                                        z: ORBIT_DISTANCE * Math.cos(orbitAngle)
                                    });
                                }

                                if (ragStore.graphData?.nodes) {
                                    ragStore.graphData.nodes.forEach((node: any) => {
                                        if (node.__threeObj && node.__baseSize) {
                                            const isLocal = node.conversation_id && node.conversation_id !== 'global';
                                            const offset = node.id ? node.id.charCodeAt(0) : 0;

                                            // Stronger pulse for local nodes
                                            const pulseIntensity = isLocal ? 0.1 : 0.05;
                                            const scale = 1 + Math.sin(time + offset) * pulseIntensity;
                                            node.__threeObj.scale.setScalar(scale);

                                            if (node.__sphereMesh && node.__sphereMesh.material) {
                                                let colorStr = node.id === highlightedNodeId ? '#ffffff' : (node.color || '#94a3b8');
                                                if (!colorStr.startsWith('#')) colorStr = '#94a3b8';

                                                const color = new THREE.Color(colorStr);
                                                if (isLocal && node.id !== highlightedNodeId) {
                                                    color.offsetHSL(0, 0.2, 0.1);
                                                }

                                                node.__sphereMesh.material.color.copy(color);

                                                if (isLocal) {
                                                    // Local nodes glow
                                                    const emissiveIntensity = 0.4 + Math.sin(time * 2 + offset) * 0.2;
                                                    node.__sphereMesh.material.emissive.copy(color);
                                                    node.__sphereMesh.material.emissiveIntensity = emissiveIntensity;
                                                } else {
                                                    node.__sphereMesh.material.emissive.setHex(node.id === highlightedNodeId ? 0x333333 : 0x000000);
                                                    node.__sphereMesh.material.emissiveIntensity = node.id === highlightedNodeId ? 0.5 : 0;
                                                }
                                            }

                                            if (node.__labelSprite) {
                                                node.__labelSprite.color = node.id === highlightedNodeId ? '#ffffff' : (isLocal ? '#f8fafc' : '#cbd5e1');
                                            }
                                        }
                                    });
                                }
                                animationFrameId = requestAnimationFrame(animate);
                            };
                            animate();

                            cleanup = () => {
                                window.removeEventListener('resize', handleResize);
                                graphContainer.removeEventListener('mousedown', handleInteraction);
                                graphContainer.removeEventListener('wheel', handleInteraction);
                                cancelAnimationFrame(animationFrameId);
                                if (graphInstance) graphInstance._destructor();
                            };
                        } catch (err) {
                            console.error('Failed to initialize ForceGraph3D:', err);
                        }
                    }, 100);
                }
            });
        }
        return () => {
            isMounted = false;
            cleanup();
        };
    });

    const handleResize = () => {
        if (graphInstance && graphContainer) {
            graphInstance.width(graphContainer.clientWidth);
            graphInstance.height(graphContainer.clientHeight);
        }
    };

    // Reactively update graph data when it changes in the store
    $effect(() => {
        if (graphInstance && ragStore.graphData) {
            graphInstance.graphData(ragStore.graphData);
        }
    });

    async function handleSearch(e: Event) {
        const target = e.target as HTMLInputElement;
        searchQuery = target.value;
        if (searchQuery.length > 1) {
            await ragStore.searchGraph(searchQuery);
            showDropdown = true;
        } else {
            ragStore.clearSearch();
            showDropdown = false;
        }
    }

    function selectResult(node: Node) {
        const fullNode = ragStore.graphData.nodes.find((n: any) => n.id === node.id);
        if (fullNode && graphInstance) {
            highlightedNodeId = fullNode.id;

            const distance = 80;
            const fn = fullNode as any;
            const distRatio = 1 + distance / Math.hypot(fn.x || 0, fn.y || 0, fn.z || 0);

            graphInstance.cameraPosition(
                {x: (fn.x || 0) * distRatio, y: (fn.y || 0) * distRatio, z: (fn.z || 0) * distRatio},
                fn,
                2000
            );
            openNodeInfo(fullNode);
        }
        showDropdown = false;
        searchQuery = node.label;
    }

    function clearSearch() {
        searchQuery = '';
        ragStore.clearSearch();
        showDropdown = false;
        highlightedNodeId = null;
    }

    function handleInteraction() {
        isInteracting = true;
        clearTimeout(interactionTimeout);

        if (graphInstance) {
            const camPos = graphInstance.cameraPosition();
            orbitAngle = Math.atan2(camPos.x, camPos.z);
        }

        interactionTimeout = setTimeout(() => {
            isInteracting = false;
        }, 5000); // Resume orbit after 5s
    }

    function resetCamera() {
        if (graphInstance) {
            handleInteraction();
            graphInstance.cameraPosition(
                {x: 0, y: 0, z: ORBIT_DISTANCE},
                {x: 0, y: 0, z: 0},
                2000
            );
        }
    }

    function openNodeInfo(node: Node) {
        selectedNode = node;
        highlightedNodeId = node.id;
        popupStore.open({
            title: 'Entity Details',
            width: 'w-full max-w-lg',
            contentSnippet: nodeInfoSnippet
        });
    }

</script>

{#snippet nodeInfoSnippet()}
    {#if selectedNode}
        <div class="space-y-8 animate-in fade-in slide-in-from-right-4 duration-300">
            <div class="space-y-1">
                <div class="flex items-center gap-2">
                    <span class="px-2 py-0.5 rounded-md text-[10px] font-black uppercase tracking-widest bg-emerald-500/10 text-emerald-500 border border-emerald-500/20">
                        {selectedNode.node_type}
                    </span>
                    {#if selectedNode.conversation_id && selectedNode.conversation_id !== 'global'}
                        <span class="px-2 py-0.5 rounded-md text-[10px] font-black uppercase tracking-widest bg-blue-500/10 text-blue-400 border border-blue-500/20">
                            Current Soul
                        </span>
                    {/if}
                </div>
                <h2 class="text-3xl font-black text-zinc-100 tracking-tight">{selectedNode.label}</h2>
            </div>

            <div class="grid gap-6">
                <div class="bg-zinc-900/30 border border-zinc-800/50 rounded-2xl p-6 space-y-4">
                    <div class="flex items-center gap-2 text-zinc-400">
                        <span class="text-xs font-bold uppercase tracking-wider">Entity Details</span>
                    </div>
                    
                    <div class="space-y-4 overflow-x-auto">
                        <div class="prose prose-invert prose-sm max-w-none text-zinc-400">
                            {#if mdIt}
                                {@html mdIt.render(selectedNode.label || '')}
                            {:else}
                                <p class="leading-relaxed">
                                    {selectedNode.label}
                                </p>
                            {/if}
                            
                            <div class="mt-6 grid grid-cols-2 gap-4">
                                <div class="p-4 rounded-xl bg-zinc-950 border border-zinc-900">
                                    <p class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-1">Node ID</p>
                                    <p class="text-xs font-mono text-zinc-300 truncate">{selectedNode.id}</p>
                                </div>
                                <div class="p-4 rounded-xl bg-zinc-950 border border-zinc-900">
                                    <p class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-1">Type</p>
                                    <p class="text-xs font-bold text-emerald-500 capitalize">{selectedNode.node_type}</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="space-y-4">
                    <div class="flex items-center gap-2 text-zinc-500 px-2">
                        <span class="text-[10px] font-bold uppercase tracking-widest">Related Connections</span>
                    </div>
                    <p class="text-sm text-zinc-500 px-2 italic">
                        Connections are being calculated based on semantic proximity and co-occurrence in your chat history.
                    </p>
                </div>
            </div>
        </div>
    {/if}
{/snippet}

<main class="flex-1 flex flex-col relative overflow-hidden bg-slate-950">
    <div bind:this={graphContainer} class="w-full h-full cursor-grab active:cursor-grabbing"></div>

    <!-- Floating Search Bar -->
    <div class="absolute top-4 left-4 w-[calc(100%-2rem)] sm:w-80 z-30">
        <div class="relative group">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                {#if ragStore.isSearching}
                    <Loader2 class="h-4 w-4 text-zinc-500 animate-spin"/>
                {:else}
                    <Search class="h-4 w-4 text-zinc-500 group-focus-within:text-emerald-500 transition-colors"/>
                {/if}
            </div>
            <input
                    type="text"
                    bind:value={searchQuery}
                    oninput={handleSearch}
                    onfocus={() => searchQuery.length > 1 && (showDropdown = true)}
                    placeholder="Search entities..."
                    class="block w-full pl-10 pr-10 py-2.5 bg-zinc-900/90 border border-zinc-800 focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/20 text-zinc-100 placeholder-zinc-500 rounded-xl backdrop-blur-md shadow-2xl transition-all outline-none text-sm"
            />
            {#if searchQuery}
                <button
                        onclick={clearSearch}
                        class="absolute inset-y-0 right-0 pr-3 flex items-center text-zinc-500 hover:text-zinc-300 transition-colors"
                >
                    <X class="h-4 w-4"/>
                </button>
            {/if}
        </div>

        {#if showDropdown && ragStore.searchResults.length > 0}
            <div class="absolute mt-2 w-full bg-zinc-900/95 border border-zinc-800 rounded-xl shadow-2xl backdrop-blur-md overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200">
                <div class="max-h-64 overflow-y-auto p-1">
                    {#each ragStore.searchResults as result}
                        <button
                                onclick={() => selectResult(result)}
                                class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-emerald-500/10 text-left rounded-lg transition-colors group"
                        >
                            <div class="w-2 h-2 rounded-full"
                                 style="background-color: {result.color || '#94a3b8'}"></div>
                            <div class="flex-1 min-w-0">
                                <p class="text-sm font-medium text-zinc-100 truncate">{result.label}</p>
                                <p class="text-[10px] text-zinc-500 uppercase tracking-wider">{result.node_type}</p>
                            </div>
                        </button>
                    {/each}
                </div>
            </div>
        {:else if showDropdown && searchQuery.length > 1 && !ragStore.isSearching}
            <div class="absolute mt-2 w-full bg-zinc-900/95 border border-zinc-800 rounded-xl p-4 shadow-2xl backdrop-blur-md text-center">
                <p class="text-xs text-zinc-500">No results found for "{searchQuery}"</p>
            </div>
        {/if}
    </div>

    <!-- Controls -->
    <div class="absolute top-4 right-4 flex items-center gap-2 z-10">
        <div class="bg-zinc-900/80 border border-zinc-800 backdrop-blur-sm rounded-xl p-1 flex items-center">
            <button
                    onclick={resetCamera}
                    class="p-2 hover:bg-zinc-800 rounded-lg text-zinc-400 hover:text-zinc-100 transition-all"
                    title="Reset Camera"
            >
                <Camera class="w-4 h-4"/>
            </button>
            <button
                    onclick={() => ragStore.fetchGraph()}
                    disabled={ragStore.loading}
                    class="p-2 hover:bg-zinc-800 rounded-lg text-zinc-400 hover:text-zinc-100 transition-all disabled:opacity-50"
                    title="Refresh Graph"
            >
                <RefreshCw class="w-4 h-4 {ragStore.loading ? 'animate-spin' : ''}"/>
            </button>
        </div>
    </div>

    <!-- Legend (Hidden on mobile) -->
    <div class="absolute left-6 bottom-6 p-4 bg-zinc-900/80 border border-zinc-800 backdrop-blur-sm rounded-xl space-y-2 z-10 pointer-events-none hidden lg:block">
        <h3 class="text-[10px] font-bold uppercase tracking-wider text-zinc-500 mb-2">Recent Entities</h3>
        {#each ragStore.graphData.nodes.filter((v,idx,)=>idx < 6) as row}
            <div class="flex items-center gap-3">
                <div class="w-2.5 h-2.5 rounded-full" style="background-color: {row.color}"></div>
                <span class="text-[11px] text-zinc-300 truncate max-w-[120px]">{row.label}</span>
            </div>
        {/each}
    </div>

    {#if ragStore.loading}
        <div class="absolute inset-0 flex items-center justify-center bg-slate-950/50 backdrop-blur-sm z-50">
            <div class="flex flex-col items-center gap-4">
                <div class="w-12 h-12 border-4 border-emerald-500 border-t-transparent rounded-full animate-spin"></div>
                <p class="text-slate-400 font-medium">Extracting Knowledge Graph...</p>
            </div>
        </div>
    {/if}

    {#if ragStore.error}
        <div class="absolute inset-0 flex items-center justify-center bg-slate-950/50 backdrop-blur-sm z-50">
            <div class="bg-zinc-900 border border-zinc-800 p-6 rounded-2xl max-w-sm text-center space-y-4">
                <AlertCircle class="w-12 h-12 text-red-500 mx-auto"/>
                <h2 class="text-zinc-100 font-bold">Failed to Load Graph</h2>
                <p class="text-zinc-500 text-sm">{ragStore.error}</p>
                <button
                        onclick={() => ragStore.fetchGraph()}
                        class="w-full py-2 px-4 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 rounded-xl transition-colors flex items-center justify-center gap-2"
                >
                    <RefreshCw class="w-4 h-4"/>
                    Retry
                </button>
            </div>
        </div>
    {/if}
</main>

<style>
    :global(canvas) {
        display: block;
        outline: none;
    }
</style>
