<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { X, Info, RefreshCw, AlertCircle, Search, Loader2, Camera } from 'lucide-svelte';
	import { ragStore, type Node } from '$lib/stores/rag.svelte';
	import * as THREE from 'three';

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

	onMount(() => {
		let isMounted = true;
		let cleanup = () => {};

		Promise.all([
			import('3d-force-graph'),
			import('three-spritetext')
		]).then(([module, spriteTextModule]) => {
			if (!isMounted) return;

			const ForceGraph3D = module.default || module;
			const SpriteText = spriteTextModule.default || spriteTextModule;
			ragStore.fetchGraph();

			if (graphContainer) {
				setTimeout(() => {
					try {
						graphInstance = (ForceGraph3D as any)()(graphContainer)
							.backgroundColor('#020617') // slate-950 deep black
							.nodeId('id')
							.nodeLabel((node: any) => `${node.label} (${node.node_type})`)
							.nodeThreeObject((node: any) => {
								const isSummary = node.node_type?.toLowerCase() === 'summary';
								const size = isSummary ? 12 : 5;
								const color = node.id === highlightedNodeId ? '#ffffff' : (node.color || '#94a3b8');

								const material = new THREE.MeshPhongMaterial({
									color: color,
									transparent: true,
									opacity: 0.9,
									shininess: 100
								});

								const geometry = new THREE.SphereGeometry(size, 32, 32);
								const mesh = new THREE.Mesh(geometry, material);

								node.__sphereMesh = mesh;
								node.__baseSize = size;

								if (isSummary) {
									const group = new THREE.Group();
									group.add(mesh);

									const sprite = new (SpriteText as any)(node.label);
									sprite.color = '#ffffff';
									sprite.textHeight = 6;
									sprite.position.y = size + 8;
									group.add(sprite);

									node.__threeObj = group;
									return group;
								}

								node.__threeObj = mesh;
								return mesh;
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
								selectedNode = node;
								highlightedNodeId = node.id;
								handleInteraction();
								const distance = 80;
								const distRatio = 1 + distance / Math.hypot(node.x || 0, node.y || 0, node.z || 0);
								graphInstance.cameraPosition(
									{ x: (node.x || 0) * distRatio, y: (node.y || 0) * distRatio, z: (node.z || 0) * distRatio },
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
										const offset = node.id ? node.id.charCodeAt(0) : 0;
										const scale = 1 + Math.sin(time + offset) * 0.05;
										node.__threeObj.scale.setScalar(scale);

										if (node.__sphereMesh && node.__sphereMesh.material) {
											const targetColor = node.id === highlightedNodeId ? 0xffffff : parseInt((node.color || '#94a3b8').replace('#', '0x'));
											node.__sphereMesh.material.color.setHex(targetColor);
											node.__sphereMesh.material.emissive.setHex(node.id === highlightedNodeId ? 0x333333 : 0x000000);
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
			const distRatio = 1 + distance/Math.hypot(fn.x || 0, fn.y || 0, fn.z || 0);

			graphInstance.cameraPosition(
				{ x: (fn.x || 0) * distRatio, y: (fn.y || 0) * distRatio, z: (fn.z || 0) * distRatio },
				fn,
				2000
			);
			selectedNode = fullNode;
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
				{ x: 0, y: 0, z: ORBIT_DISTANCE },
				{ x: 0, y: 0, z: 0 },
				2000
			);
		}
	}
</script>

<main class="flex-1 flex flex-col relative overflow-hidden bg-slate-950">
	<div bind:this={graphContainer} class="flex-1 w-full h-full cursor-grab active:cursor-grabbing"></div>

	<!-- Floating Search Bar -->
	<div class="absolute top-4 left-4 w-80 z-30">
		<div class="relative group">
			<div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
				{#if ragStore.isSearching}
					<Loader2 class="h-4 w-4 text-zinc-500 animate-spin" />
				{:else}
					<Search class="h-4 w-4 text-zinc-500 group-focus-within:text-emerald-500 transition-colors" />
				{/if}
			</div>
			<input
				type="text"
				bind:value={searchQuery}
				oninput={handleSearch}
				onfocus={() => searchQuery.length > 1 && (showDropdown = true)}
				placeholder="Search entities or topics..."
				class="block w-full pl-10 pr-10 py-2.5 bg-zinc-900/90 border border-zinc-800 focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/20 text-zinc-100 placeholder-zinc-500 rounded-xl backdrop-blur-md shadow-2xl transition-all outline-none text-sm"
			/>
			{#if searchQuery}
				<button
					onclick={clearSearch}
					class="absolute inset-y-0 right-0 pr-3 flex items-center text-zinc-500 hover:text-zinc-300 transition-colors"
				>
					<X class="h-4 w-4" />
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
							<div class="w-2 h-2 rounded-full" style="background-color: {result.color || '#94a3b8'}"></div>
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

	{#if ragStore.loading}
		<div class="absolute inset-0 flex items-center justify-center bg-slate-950/50 backdrop-blur-sm z-10">
			<div class="flex flex-col items-center gap-4">
				<div class="w-12 h-12 border-4 border-emerald-500 border-t-transparent rounded-full animate-spin"></div>
				<p class="text-slate-400 font-medium">Extracting Knowledge Graph...</p>
			</div>
		</div>
	{/if}

	{#if ragStore.error}
		<div class="absolute inset-0 flex items-center justify-center bg-slate-950/50 backdrop-blur-sm z-10">
			<div class="bg-zinc-900 border border-zinc-800 p-6 rounded-2xl max-w-sm text-center space-y-4">
				<AlertCircle class="w-12 h-12 text-red-500 mx-auto" />
				<h2 class="text-zinc-100 font-bold">Failed to Load Graph</h2>
				<p class="text-zinc-500 text-sm">{ragStore.error}</p>
				<button
					onclick={() => ragStore.fetchGraph()}
					class="w-full py-2 px-4 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 rounded-xl transition-colors flex items-center justify-center gap-2"
				>
					<RefreshCw class="w-4 h-4" />
					Retry
				</button>
			</div>
		</div>
	{/if}

	<!-- Header/Controls -->
	<div class="absolute top-4 right-4 flex items-center gap-4 z-10">
		<div class="bg-zinc-900/80 border border-zinc-800 backdrop-blur-sm rounded-xl p-1.5 flex items-center gap-1">
			<button
				onclick={resetCamera}
				class="p-2 hover:bg-zinc-800 rounded-lg text-zinc-400 hover:text-zinc-100 transition-all"
				title="Reset Camera"
			>
				<Camera class="w-4 h-4" />
			</button>
			<button
				onclick={() => ragStore.fetchGraph()}
				disabled={ragStore.loading}
				class="p-2 hover:bg-zinc-800 rounded-lg text-zinc-400 hover:text-zinc-100 transition-all disabled:opacity-50"
				title="Refresh Graph"
			>
				<RefreshCw class="w-4 h-4 {ragStore.loading ? 'animate-spin' : ''}" />
			</button>
		</div>
	</div>

	<!-- Side Drawer for Node Info -->
	{#if selectedNode}
		<div class="absolute right-0 top-0 bottom-0 w-80 bg-zinc-900/90 border-l border-zinc-800 backdrop-blur-md p-6 shadow-2xl transition-transform transform translate-x-0 z-20">
			<div class="flex justify-between items-start mb-6">
				<div class="flex items-center gap-2">
					<Info class="w-5 h-5 text-emerald-500" />
					<h2 class="text-lg font-bold text-zinc-100">Entity Details</h2>
				</div>
				<button onclick={() => selectedNode = null} class="p-1 hover:bg-zinc-800 rounded-lg transition-colors">
					<X class="w-5 h-5 text-zinc-400" />
				</button>
			</div>

			<div class="space-y-6">
				<div>
					<label class="text-[10px] font-bold uppercase tracking-wider text-zinc-500">Label</label>
					<p class="text-xl font-semibold text-zinc-100">{selectedNode.label}</p>
				</div>

				<div>
					<label class="text-[10px] font-bold uppercase tracking-wider text-zinc-500">Type</label>
					<div class="mt-1">
						<span class="px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-500 border border-emerald-500/20">
							{selectedNode.node_type}
						</span>
					</div>
				</div>

				<div>
					<label class="text-[10px] font-bold uppercase tracking-wider text-zinc-500">Node ID</label>
					<p class="text-sm font-mono text-zinc-400 mt-1">{selectedNode.id}</p>
				</div>

				<div class="pt-6 border-t border-zinc-800">
					<p class="text-xs text-zinc-500 italic">
						Entities are automatically extracted from your conversations during background summarization.
					</p>
				</div>
			</div>
		</div>
	{/if}

	<!-- Legend -->
	<div class="absolute left-6 bottom-6 p-4 bg-zinc-900/80 border border-zinc-800 backdrop-blur-sm rounded-xl space-y-2 z-10 pointer-events-none">
		<h3 class="text-[10px] font-bold uppercase tracking-wider text-zinc-500 mb-2">Legend</h3>
		<div class="flex items-center gap-3">
			<div class="w-3 h-3 rounded-full bg-[#3b82f6]"></div>
			<span class="text-xs text-zinc-300">Technology</span>
		</div>
		<div class="flex items-center gap-3">
			<div class="w-3 h-3 rounded-full bg-[#10b981]"></div>
			<span class="text-xs text-zinc-300">Project</span>
		</div>
		<div class="flex items-center gap-3">
			<div class="w-3 h-3 rounded-full bg-[#f59e0b]"></div>
			<span class="text-xs text-zinc-300">Person</span>
		</div>
	</div>
</main>

<style>
	:global(canvas) {
		display: block;
		outline: none;
	}
</style>
